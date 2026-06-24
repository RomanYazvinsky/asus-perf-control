import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mocktail/mocktail.dart';
import 'package:flutter_application_1/blocs/gpu/gpu_bloc.dart';
import 'package:flutter_application_1/blocs/gpu/gpu_event.dart';
import 'package:flutter_application_1/blocs/gpu/gpu_state.dart';
import 'package:flutter_application_1/services/gpu_service.dart';
import 'package:flutter_application_1/models/gpu_models.dart';

class MockGpuService extends Mock implements IGpuService {}

GpuConfiguration fakeGpuConfig({
  bool hasDgpu = true,
  GpuVendor? dgpuVendor = GpuVendor.nvidia,
  bool hasMuxSwitch = false,
  bool supportsEgpu = false,
  bool externalGpuConnected = false,
  bool vfioAvailable = false,
}) => GpuConfiguration(
  hasDgpu: hasDgpu,
  dgpuVendor: dgpuVendor,
  hasMuxSwitch: hasMuxSwitch,
  supportsEgpu: supportsEgpu,
  externalGpuConnected: externalGpuConnected,
  vfioAvailable: vfioAvailable,
);

GpuStatus fakeGpuStatus({
  GpuMode activeMode = GpuMode.hybrid,
  bool dgpuEnabled = true,
  double? temperature = 65.0,
  double? utilization = 45.0,
  GpuPowerState powerState = GpuPowerState.active,
}) => GpuStatus(
  activeMode: activeMode,
  dgpuEnabled: dgpuEnabled,
  temperature: temperature,
  utilization: utilization,
  powerState: powerState,
);

void main() {
  group('GpuBloc', () {
    late MockGpuService mockGpuService;

    setUpAll(() {
      registerFallbackValue(GpuMode.hybrid);
    });

    setUp(() {
      mockGpuService = MockGpuService();
    });

    blocTest<GpuBloc, GpuState>(
      'emits [GpuLoading, GpuLoaded] when LoadCapabilities succeeds with dGPU',
      build: () {
        when(
          () => mockGpuService.getCapabilities(),
        ).thenAnswer((_) async => fakeGpuConfig());
        when(
          () => mockGpuService.getStatus(),
        ).thenAnswer((_) async => fakeGpuStatus());
        when(
          () => mockGpuService.getSupportedModes(),
        ).thenAnswer((_) async => [GpuMode.integrated, GpuMode.hybrid]);
        return GpuBloc(gpuService: mockGpuService);
      },
      act: (bloc) => bloc.add(LoadCapabilitiesEvent()),
      expect: () => [
        isA<GpuLoading>(),
        isA<GpuLoaded>()
            .having((s) => s.capabilities, 'capabilities', fakeGpuConfig())
            .having((s) => s.status, 'status', fakeGpuStatus())
            .having((s) => s.supportedModes, 'supportedModes', [
              GpuMode.integrated,
              GpuMode.hybrid,
            ]),
      ],
      verify: (_) {
        verify(() => mockGpuService.getCapabilities()).called(1);
        verify(() => mockGpuService.getStatus()).called(1);
        verify(() => mockGpuService.getSupportedModes()).called(1);
      },
    );

    blocTest<GpuBloc, GpuState>(
      'emits [GpuLoading, GpuUnsupported] when no dGPU detected',
      build: () {
        when(
          () => mockGpuService.getCapabilities(),
        ).thenAnswer((_) async => fakeGpuConfig(hasDgpu: false));
        return GpuBloc(gpuService: mockGpuService);
      },
      act: (bloc) => bloc.add(LoadCapabilitiesEvent()),
      expect: () => [
        isA<GpuLoading>(),
        isA<GpuUnsupported>().having(
          (s) => s.message,
          'message',
          'GPU hardware not detected. Check that discrete GPU is present.',
        ),
      ],
    );

    blocTest<GpuBloc, GpuState>(
      'emits [GpuLoading, GpuError] when LoadCapabilities fails',
      build: () {
        when(
          () => mockGpuService.getCapabilities(),
        ).thenThrow(Exception('Failed to load'));
        return GpuBloc(gpuService: mockGpuService);
      },
      act: (bloc) => bloc.add(LoadCapabilitiesEvent()),
      expect: () => [isA<GpuLoading>(), isA<GpuError>()],
    );

    blocTest<GpuBloc, GpuState>(
      'emits updated GpuLoaded when SetModeRequested succeeds',
      build: () {
        when(() => mockGpuService.setGpuMode(any())).thenAnswer((_) async {});
        when(
          () => mockGpuService.getStatus(),
        ).thenAnswer((_) async => fakeGpuStatus(activeMode: GpuMode.ultimate));
        return GpuBloc(gpuService: mockGpuService);
      },
      seed: () => GpuLoaded(
        capabilities: fakeGpuConfig(),
        status: fakeGpuStatus(),
        supportedModes: [GpuMode.integrated, GpuMode.hybrid, GpuMode.ultimate],
      ),
      act: (bloc) => bloc.add(SetModeRequestedEvent(mode: GpuMode.ultimate)),
      expect: () => [
        isA<GpuLoaded>()
            .having((s) => s.capabilities, 'capabilities', fakeGpuConfig())
            .having(
              (s) => s.status,
              'status',
              fakeGpuStatus(activeMode: GpuMode.ultimate),
            )
            .having((s) => s.supportedModes, 'supportedModes', [
              GpuMode.integrated,
              GpuMode.hybrid,
              GpuMode.ultimate,
            ]),
      ],
    );

    blocTest<GpuBloc, GpuState>(
      'emits GpuRequiresReboot when setGpuMode throws RebootRequired',
      build: () {
        when(
          () => mockGpuService.setGpuMode(any()),
        ).thenThrow(Exception('RebootRequired'));
        return GpuBloc(gpuService: mockGpuService);
      },
      seed: () => GpuLoaded(
        capabilities: fakeGpuConfig(hasMuxSwitch: true),
        status: fakeGpuStatus(),
        supportedModes: [
          GpuMode.integrated,
          GpuMode.hybrid,
          GpuMode.asusMuxDgpu,
        ],
      ),
      act: (bloc) => bloc.add(SetModeRequestedEvent(mode: GpuMode.asusMuxDgpu)),
      expect: () => [isA<GpuRequiresReboot>()],
    );

    blocTest<GpuBloc, GpuState>(
      'emits GpuRequiresLogout when setGpuMode requires logout',
      build: () {
        when(() => mockGpuService.setGpuMode(any())).thenThrow(
          Exception('Logout required. Please save your work and logout.'),
        );
        return GpuBloc(gpuService: mockGpuService);
      },
      seed: () => GpuLoaded(
        capabilities: fakeGpuConfig(),
        status: fakeGpuStatus(),
        supportedModes: [GpuMode.integrated, GpuMode.hybrid],
      ),
      act: (bloc) => bloc.add(SetModeRequestedEvent(mode: GpuMode.integrated)),
      expect: () => [
        isA<GpuRequiresLogout>().having(
          (s) => s.message,
          'message',
          'GPU mode changed. Please save your work and log out, then log back in to finish applying the change.',
        ),
      ],
    );

    blocTest<GpuBloc, GpuState>(
      'emits GpuError when setGpuMode throws eGPU not connected',
      build: () {
        when(
          () => mockGpuService.setGpuMode(any()),
        ).thenThrow(Exception('EgpuNotConnected: eGPU not connected'));
        return GpuBloc(gpuService: mockGpuService);
      },
      seed: () => GpuLoaded(
        capabilities: fakeGpuConfig(supportsEgpu: true),
        status: fakeGpuStatus(),
        supportedModes: [GpuMode.integrated, GpuMode.hybrid, GpuMode.asusEgpu],
      ),
      act: (bloc) => bloc.add(SetModeRequestedEvent(mode: GpuMode.asusEgpu)),
      expect: () => [isA<GpuError>()],
    );

    blocTest<GpuBloc, GpuState>(
      'emits GpuError when setGpuMode throws VFIO not available',
      build: () {
        when(
          () => mockGpuService.setGpuMode(any()),
        ).thenThrow(Exception('VfioNotAvailable'));
        return GpuBloc(gpuService: mockGpuService);
      },
      seed: () => GpuLoaded(
        capabilities: fakeGpuConfig(),
        status: fakeGpuStatus(),
        supportedModes: [GpuMode.integrated, GpuMode.hybrid, GpuMode.vfio],
      ),
      act: (bloc) => bloc.add(SetModeRequestedEvent(mode: GpuMode.vfio)),
      expect: () => [isA<GpuError>()],
    );

    blocTest<GpuBloc, GpuState>(
      'calls confirmMuxAndReboot on ConfirmRebootEvent',
      build: () {
        when(
          () => mockGpuService.confirmMuxAndReboot(),
        ).thenAnswer((_) async {});
        return GpuBloc(gpuService: mockGpuService);
      },
      seed: () => GpuRequiresReboot(message: 'Reboot required'),
      act: (bloc) => bloc.add(ConfirmRebootEvent()),
      verify: (_) {
        verify(() => mockGpuService.confirmMuxAndReboot()).called(1);
      },
    );

    blocTest<GpuBloc, GpuState>(
      'emits GpuError when ConfirmRebootEvent fails',
      build: () {
        when(
          () => mockGpuService.confirmMuxAndReboot(),
        ).thenThrow(Exception('Reboot failed'));
        return GpuBloc(gpuService: mockGpuService);
      },
      seed: () => GpuRequiresReboot(message: 'Reboot required'),
      act: (bloc) => bloc.add(ConfirmRebootEvent()),
      expect: () => [isA<GpuError>()],
    );

    blocTest<GpuBloc, GpuState>(
      'emits updated status on LoadStatusEvent',
      build: () {
        when(
          () => mockGpuService.getStatus(),
        ).thenAnswer((_) async => fakeGpuStatus(temperature: 80.0));
        return GpuBloc(gpuService: mockGpuService);
      },
      seed: () => GpuLoaded(
        capabilities: fakeGpuConfig(),
        status: fakeGpuStatus(temperature: 65.0),
        supportedModes: [GpuMode.integrated, GpuMode.hybrid],
      ),
      act: (bloc) => bloc.add(LoadStatusEvent()),
      expect: () => [
        isA<GpuLoaded>()
            .having((s) => s.capabilities, 'capabilities', fakeGpuConfig())
            .having((s) => s.status, 'status', fakeGpuStatus(temperature: 80.0))
            .having((s) => s.supportedModes, 'supportedModes', [
              GpuMode.integrated,
              GpuMode.hybrid,
            ]),
      ],
    );
  });
}
