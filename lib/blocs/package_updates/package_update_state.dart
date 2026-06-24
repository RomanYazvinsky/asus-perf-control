import '../../services/package_update_service.dart';

class PackageUpdateState {
  const PackageUpdateState({
    this.status,
    this.backend,
    this.updateId,
    this.isChecking = false,
    this.daemonUnavailable = false,
    this.lastError,
  });

  final PackageUpdateStatus? status;
  final PackageUpdateBackend? backend;
  final String? updateId;
  final bool isChecking;
  final bool daemonUnavailable;
  final String? lastError;

  bool get isUpdateRunning => status?.state == PackageUpdateRunState.running;

  bool get canStartUpdate =>
      !daemonUnavailable &&
      !isChecking &&
      status?.state == PackageUpdateRunState.available;

  String get summary {
    if (daemonUnavailable) {
      return 'Package update daemon is unavailable';
    }
    final currentStatus = status;
    if (currentStatus == null) {
      return 'Package update status is not loaded';
    }
    if (currentStatus.rebootRequired == true) {
      return 'Restart required to finish updating';
    }
    if (currentStatus.state == PackageUpdateRunState.running) {
      return 'Package update is running';
    }
    if (currentStatus.state == PackageUpdateRunState.upToDate) {
      return 'ASUS Linux Control is up to date';
    }
    if (currentStatus.state == PackageUpdateRunState.unsupported) {
      return 'Package updates are not supported on this system';
    }
    if (currentStatus.state == PackageUpdateRunState.failed) {
      return _errorSummary(currentStatus.errorCode);
    }
    return currentStatus.summary;
  }

  PackageUpdateState copyWith({
    PackageUpdateStatus? status,
    PackageUpdateBackend? backend,
    String? updateId,
    bool? isChecking,
    bool? daemonUnavailable,
    String? lastError,
    bool clearUpdateId = false,
  }) {
    return PackageUpdateState(
      status: status ?? this.status,
      backend: backend ?? this.backend,
      updateId: clearUpdateId ? null : updateId ?? this.updateId,
      isChecking: isChecking ?? this.isChecking,
      daemonUnavailable: daemonUnavailable ?? this.daemonUnavailable,
      lastError: lastError ?? this.lastError,
    );
  }
}

String _errorSummary(PackageUpdateErrorCode? errorCode) {
  switch (errorCode) {
    case PackageUpdateErrorCode.missingRepository:
      return 'Package repository is not configured';
    case PackageUpdateErrorCode.signatureError:
      return 'Package signature verification failed';
    case PackageUpdateErrorCode.packageDatabaseLocked:
      return 'Package database is locked';
    case PackageUpdateErrorCode.unsupportedOs:
      return 'Package updates are not supported on this system';
    case PackageUpdateErrorCode.authorizationDenied:
      return 'Authorization was denied';
    case PackageUpdateErrorCode.networkError:
      return 'Network error while checking updates';
    case PackageUpdateErrorCode.packageManagerUnavailable:
      return 'Package manager is unavailable';
    case PackageUpdateErrorCode.rebootRequired:
      return 'Restart required to finish updating';
    case PackageUpdateErrorCode.updateFailed:
    case null:
      return 'Package update failed';
  }
}
