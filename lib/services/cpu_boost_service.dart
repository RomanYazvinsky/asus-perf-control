enum CpuBoostMode {
  disabled('disabled', 'Disabled'),
  efficient('efficient', 'Efficient'),
  efficientEnabled('efficient_enabled', 'Efficient Enabled'),
  enabled('enabled', 'Enabled'),
  efficientAggressive('efficient_aggressive', 'Efficient Aggressive'),
  aggressive('aggressive', 'Aggressive');

  const CpuBoostMode(this.daemonId, this.label);

  final String daemonId;
  final String label;

  static CpuBoostMode fromDaemonId(String id) {
    return CpuBoostMode.values.firstWhere(
      (mode) => mode.daemonId == id,
      orElse: () => CpuBoostMode.enabled,
    );
  }
}

class CpuBoostStatus {
  final bool isSupported;
  final CpuBoostMode currentMode;
  final bool boostEnabled;
  final String? epp;
  final bool eppWritable;
  final List<CpuBoostMode> availableModes;

  const CpuBoostStatus({
    required this.isSupported,
    required this.currentMode,
    required this.boostEnabled,
    required this.availableModes,
    this.epp,
    this.eppWritable = true,
  });
}

abstract class ICpuBoostService {
  Future<CpuBoostStatus> getCpuBoostStatus();
  Future<void> setCpuBoostMode(CpuBoostMode mode);
}
