import 'dart:io';

import 'package:flutter/material.dart';
import 'screens/battery_screen.dart';
import 'screens/fan_screen.dart';
import 'screens/gpu_screen.dart';
import 'screens/home_screen.dart';
import 'screens/package_updates_screen.dart';
import 'screens/profiles_screen.dart';
import 'screens/tdp_screen.dart';
import 'screens/xg_mobile_fan_diagnostic_screen.dart';
import 'bridge_generated.dart/frb_generated.dart';
import 'services/bridge_service.dart';
import 'services/backend_logger.dart';
import 'services/input/steam_input_service.dart';
import 'services/settings/app_settings_store.dart';
import 'services/settings/hardware_profile_store.dart';
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
  final AppSettingsStore? settingsStore;
  final HardwareProfileStore? profileStore;

  const AsusLinuxControlApp({super.key, this.settingsStore, this.profileStore});

  @override
  Widget build(BuildContext context) {
    final globalSettingsStore = settingsStore ?? FileAppSettingsStore();
    final effectiveProfileStore =
        profileStore ?? _defaultProfileStore(globalSettingsStore);
    final effectiveSettingsStore =
        globalSettingsStore is ProfiledAppSettingsStore
        ? globalSettingsStore
        : ProfiledAppSettingsStore(
            globalStore: globalSettingsStore,
            profileStore: effectiveProfileStore,
          );

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
      home: AppShell(
        settingsStore: effectiveSettingsStore,
        profileStore: effectiveProfileStore,
      ),
    );
  }
}

class AppShell extends StatefulWidget {
  final AppSettingsStore? settingsStore;
  final HardwareProfileStore? profileStore;
  final VoidCallback? onProfileReloadForTest;

  const AppShell({
    super.key,
    this.settingsStore,
    this.profileStore,
    this.onProfileReloadForTest,
  });

  @override
  State<AppShell> createState() => _AppShellState();
}

class _AppShellState extends State<AppShell> with WidgetsBindingObserver {
  int _selectedIndex = 0;
  int _profileReloadToken = 0;
  late final AppSettingsStore _settingsStore;
  late final HardwareProfileStore _profileStore;
  late List<Widget> _screens;

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
    NavigationRailDestination(
      icon: Icon(Icons.manage_accounts_outlined),
      selectedIcon: Icon(Icons.manage_accounts),
      label: Text('Profiles'),
    ),
  ];

  @override
  void initState() {
    super.initState();
    final globalSettingsStore = widget.settingsStore ?? FileAppSettingsStore();
    _profileStore =
        widget.profileStore ?? _defaultProfileStore(globalSettingsStore);
    _settingsStore = globalSettingsStore is ProfiledAppSettingsStore
        ? globalSettingsStore
        : ProfiledAppSettingsStore(
            globalStore: globalSettingsStore,
            profileStore: _profileStore,
          );
    _screens = _buildScreens();
    WidgetsBinding.instance.addObserver(this);
    _loadSavedSettings();
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
            onDestinationSelected: _selectDestination,
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

  Future<void> _loadSavedSettings() async {
    final settings = await _settingsStore.load();
    final savedIndex = settings.ui.selectedNavigationIndex;
    if (savedIndex < 0 || savedIndex >= _destinations.length) {
      BackendLogger.logEvent(
        'Settings.readMismatch',
        fields: {
          'key': 'ui.selectedNavigationIndex',
          'actual': savedIndex,
          'expected': '0..${_destinations.length - 1}',
        },
      );
      return;
    }
    if (!mounted) return;
    setState(() => _selectedIndex = savedIndex);
  }

  void _selectDestination(int index) {
    setState(() => _selectedIndex = index);
    _settingsStore.update((settings) {
      return settings.copyWith(
        ui: settings.ui.copyWith(selectedNavigationIndex: index),
      );
    });
  }

  List<Widget> _buildScreens() {
    return [
      HomeScreen(
        key: ValueKey('home_$_profileReloadToken'),
        settingsStore: _settingsStore,
      ),
      TdpScreen(
        key: ValueKey('tdp_$_profileReloadToken'),
        settingsStore: _settingsStore,
      ),
      GpuScreen(
        key: ValueKey('gpu_$_profileReloadToken'),
        settingsStore: _settingsStore,
      ),
      FanScreen(
        key: ValueKey('fan_$_profileReloadToken'),
        settingsStore: _settingsStore,
      ),
      BatteryScreen(
        key: ValueKey('battery_$_profileReloadToken'),
        settingsStore: _settingsStore,
      ),
      XgMobileFanDiagnosticScreen(),
      const PackageUpdatesScreen(),
      ProfilesScreen(
        settingsStore: _settingsStore,
        profileStore: _profileStore,
        onProfileApplied: _reloadHardwareScreensForProfile,
      ),
    ];
  }

  void _reloadHardwareScreensForProfile() {
    setState(() {
      _profileReloadToken += 1;
      _screens = _buildScreens();
    });
    widget.onProfileReloadForTest?.call();
  }
}

FileHardwareProfileStore _defaultProfileStore(AppSettingsStore settingsStore) {
  final settingsFile = settingsStore is FileAppSettingsStore
      ? settingsStore.file
      : FileAppSettingsStore.defaultSettingsFile();
  return FileHardwareProfileStore(
    directory: Directory('${settingsFile.parent.path}/profiles'),
  );
}
