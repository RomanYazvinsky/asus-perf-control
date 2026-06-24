abstract class PackageUpdateEvent {}

class LoadPackageUpdateStatus extends PackageUpdateEvent {}

class CheckPackageUpdatesRequested extends PackageUpdateEvent {}

class StartPackageUpdateRequested extends PackageUpdateEvent {}

class PollPackageUpdateStatusRequested extends PackageUpdateEvent {}

class CancelPackageUpdateRequested extends PackageUpdateEvent {}
