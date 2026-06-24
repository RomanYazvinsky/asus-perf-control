// Battery service interface
// Abstracts FFI calls for battery charge limit control

/// Data class for battery status
class BatteryStatus {
  final int chargeLimit;
  final double? percentage;
  final String? state;

  BatteryStatus({required this.chargeLimit, this.percentage, this.state});
}

/// Interface for battery operations (allows mocking in tests)
abstract class IBatteryService {
  /// Get current charge limit (0-100, 0 = no limit)
  Future<int> getChargeLimit();

  /// Set charge limit (0-100, 0 = disable limit)
  Future<void> setChargeLimit(int limit);

  /// Get battery status info
  Future<BatteryStatus> getBatteryStatus();

  /// Check if charge limit control is supported
  Future<bool> isChargeLimitSupported();
}
