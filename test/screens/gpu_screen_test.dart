import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:flutter_application_1/blocs/gpu/gpu_bloc.dart';
import 'package:flutter_application_1/blocs/gpu/gpu_state.dart';
import 'package:flutter_application_1/models/gpu_models.dart';
import 'package:flutter_application_1/screens/gpu_screen.dart';
import 'package:flutter_application_1/services/gpu_service.dart';

class _NoopGpuService implements IGpuService {
  @override
  Future<void> confirmMuxAndReboot() async {}

  @override
  Future<GpuConfiguration> getCapabilities() async => const GpuConfiguration(
    hasDgpu: true,
    dgpuVendor: GpuVendor.nvidia,
    hasMuxSwitch: true,
    supportsEgpu: false,
    externalGpuConnected: false,
    vfioAvailable: false,
  );

  @override
  Future<GpuStatus> getStatus() async => const GpuStatus(
    activeMode: GpuMode.hybrid,
    dgpuEnabled: true,
    powerState: GpuPowerState.active,
  );

  @override
  Future<List<GpuMode>> getSupportedModes() async => [GpuMode.hybrid];

  @override
  Future<void> rebindDgpuToGraphics() async {}

  @override
  Future<void> setGpuMode(GpuMode mode) async {}
}

class _RebootStateGpuBloc extends GpuBloc {
  _RebootStateGpuBloc() : super(gpuService: _NoopGpuService());

  @override
  GpuState get state => GpuLoading();

  @override
  Stream<GpuState> get stream => Stream<GpuState>.value(
    GpuRequiresReboot(message: 'Reboot required for this mode change'),
  );
}

void main() {
  group('GpuScreen dialog behavior', () {
    testWidgets('reboot-required dialog has no non-functional reboot action', (
      tester,
    ) async {
      final bloc = _RebootStateGpuBloc();

      await tester.pumpWidget(
        MaterialApp(
          home: BlocProvider<GpuBloc>.value(
            value: bloc,
            child: const GpuView(),
          ),
        ),
      );
      await tester.pump(const Duration(milliseconds: 200));

      expect(find.text('Reboot Required'), findsOneWidget);
      expect(find.text('Reboot Now'), findsNothing);
      expect(find.text('OK'), findsOneWidget);

      await tester.tap(find.text('OK'));
      await tester.pump(const Duration(milliseconds: 200));

      expect(find.text('Reboot Required'), findsNothing);
    });
  });
}
