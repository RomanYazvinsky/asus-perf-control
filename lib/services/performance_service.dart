abstract class IPerformanceService {
  Future<int> getPerformanceMode();
  Future<void> setPerformanceMode(int mode);
}
