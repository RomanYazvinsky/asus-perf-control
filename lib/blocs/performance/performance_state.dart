// Performance states for BLoC pattern

abstract class PerformanceState {}

/// Initial state before any data is loaded
class PerformanceInitial extends PerformanceState {}

/// Loading state while fetching current mode
class PerformanceLoading extends PerformanceState {}

/// Successfully loaded current mode
class PerformanceLoaded extends PerformanceState {
  final int currentMode;
  final String modeName;

  PerformanceLoaded(this.currentMode) : modeName = _getModeName(currentMode);

  static String _getModeName(int mode) {
    switch (mode) {
      case 0:
        return 'Silent';
      case 1:
        return 'Balanced';
      case 2:
        return 'Turbo';
      default:
        return 'Unknown';
    }
  }
}

/// Error state with error message
class PerformanceError extends PerformanceState {
  final String error;
  final String message;

  PerformanceError(this.error) : message = error;
}
