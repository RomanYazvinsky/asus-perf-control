import 'package:flutter/material.dart';
import 'screens/battery_screen.dart';
import 'screens/fan_screen.dart';
import 'screens/gpu_screen.dart';
import 'screens/home_screen.dart';
import 'screens/package_updates_screen.dart';
import 'screens/tdp_screen.dart';
import 'screens/xg_mobile_fan_diagnostic_screen.dart';
import 'bridge_generated.dart/frb_generated.dart';
import 'services/bridge_service.dart';
import 'services/input/steam_input_service.dart';
import 'widgets/copyable_alert.dart';
import 'widgets/input_diagnostics.dart';
import 'widgets/input/gamepad_input_scope.dart';

final GlobalKey<NavigatorState> navigatorKey = GlobalKey<NavigatorState>();
final SteamInputService steamInputService = FfiSteamInputService();

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  await RustLib.init();
  runApp(const AsusLinuxControlApp());
}

class AsusLinuxControlApp extends StatelessWidget {
  const AsusLinuxControlApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'ASUS Linux Control',
      navigatorKey: navigatorKey,
      theme: ThemeData(
        useMaterial3: true,
        brightness: Brightness.dark,
        colorScheme: ColorScheme.fromSeed(
          seedColor: Colors.blue,
          brightness: Brightness.dark,
        ),
      ),
      builder: (context, child) {
        ErrorWidget.builder = (FlutterErrorDetails details) {
          // Show error via CopyableAlert with copy support
          WidgetsBinding.instance.addPostFrameCallback((_) {
            final ctx = navigatorKey.currentContext;
            if (ctx != null) {
              CopyableAlert.showError(
                ctx,
                title: 'Application Error',
                message: 'An unexpected error occurred in the UI',
                errorDetails: details.exception.toString(),
              );
            }
          });
          return const SizedBox.shrink();
        };
        return GamepadInputScope(
          service: steamInputService,
          child: InputDiagnostics(
            enabled: const bool.fromEnvironment('ASUS_INPUT_DIAGNOSTICS'),
            child: child ?? const SizedBox.shrink(),
          ),
        );
      },
      home: const AppShell(),
    );
  }
}

class AppShell extends StatefulWidget {
  const AppShell({super.key});

  @override
  State<AppShell> createState() => _AppShellState();
}

class _AppShellState extends State<AppShell> with WidgetsBindingObserver {
  int _selectedIndex = 0;

  static const List<NavigationRailDestination> _destinations = [
    NavigationRailDestination(
      icon: Icon(Icons.speed_outlined),
      selectedIcon: Icon(Icons.speed),
      label: Text('Performance'),
    ),
    NavigationRailDestination(
      icon: Icon(Icons.power_settings_new_outlined),
      selectedIcon: Icon(Icons.power_settings_new),
      label: Text('TDP'),
    ),
    NavigationRailDestination(
      icon: Icon(Icons.memory_outlined),
      selectedIcon: Icon(Icons.memory),
      label: Text('GPU'),
    ),
    NavigationRailDestination(
      icon: Icon(Icons.air_outlined),
      selectedIcon: Icon(Icons.air),
      label: Text('Fans'),
    ),
    NavigationRailDestination(
      icon: Icon(Icons.battery_charging_full_outlined),
      selectedIcon: Icon(Icons.battery_charging_full),
      label: Text('Battery'),
    ),
    NavigationRailDestination(
      icon: Icon(Icons.cable_outlined),
      selectedIcon: Icon(Icons.cable),
      label: Text('XG Diag'),
    ),
    NavigationRailDestination(
      icon: Icon(Icons.system_update_alt_outlined),
      selectedIcon: Icon(Icons.system_update_alt),
      label: Text('Updates'),
    ),
  ];

  static final List<Widget> _screens = [
    HomeScreen(),
    TdpScreen(),
    GpuScreen(),
    FanScreen(),
    BatteryScreen(),
    XgMobileFanDiagnosticScreen(),
    PackageUpdatesScreen(),
  ];

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addObserver(this);
  }

  @override
  void dispose() {
    WidgetsBinding.instance.removeObserver(this);
    BridgeService().dispose();
    super.dispose();
  }

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    if (state == AppLifecycleState.detached) {
      BridgeService().dispose();
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: Row(
        children: [
          NavigationRail(
            selectedIndex: _selectedIndex,
            onDestinationSelected: (index) =>
                setState(() => _selectedIndex = index),
            labelType: NavigationRailLabelType.all,
            destinations: _destinations,
          ),
          const VerticalDivider(thickness: 1, width: 1),
          Expanded(
            child: IndexedStack(index: _selectedIndex, children: _screens),
          ),
        ],
      ),
    );
  }
}
