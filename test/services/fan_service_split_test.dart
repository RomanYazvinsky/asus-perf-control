import 'package:flutter_test/flutter_test.dart';

import 'package:flutter_application_1/services/fan/fan_models.dart'
    as split_models;
import 'package:flutter_application_1/services/fan/fan_service_contract.dart'
    as split_contract;
import 'package:flutter_application_1/services/fan/fan_service_impl.dart'
    as split_impl;
import 'package:flutter_application_1/services/fan_service.dart';
import 'package:flutter_application_1/services/asusd_fan_client.dart';

void main() {
  test('fan service split modules remain compatible with shim export', () {
    final config = split_models.FanEngineConfig(
      cpuCurve: [FanCurvePoint(tempC: 60, speedPercent: 50)],
      gpuCurve: [FanCurvePoint(tempC: 60, speedPercent: 50)],
    );

    expect(config.tickMs, 1000);
    expect(split_models.FanId.cpu.name, 'cpu');

    split_contract.IFanService splitService = split_impl.FanServiceImpl();
    IFanService shimService = split_impl.FanServiceImpl();

    expect(splitService, isA<split_contract.IFanService>());
    expect(shimService, isA<IFanService>());
  });
}
