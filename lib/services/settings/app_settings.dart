class AppSettings {
  static const int currentSchemaVersion = 2;
  static const defaults = AppSettings();

  final int schemaVersion;
  final UiSettings ui;
  final HardwareSettings hardware;

  const AppSettings({
    this.schemaVersion = currentSchemaVersion,
    this.ui = const UiSettings(),
    this.hardware = const HardwareSettings(),
  });

  factory AppSettings.fromJson(
    Map<String, dynamic> json, {
    List<String>? issues,
  }) {
    final schemaVersion = _readInt(
      json,
      'schemaVersion',
      defaultValue: currentSchemaVersion,
      issues: issues,
    );
    return AppSettings(
      schemaVersion: schemaVersion,
      ui: UiSettings.fromJson(_readMap(json, 'ui', issues: issues), issues),
      hardware: HardwareSettings.fromJson(
        _readMap(json, 'hardware', issues: issues),
        issues,
      ),
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'schemaVersion': schemaVersion,
      'ui': ui.toJson(),
      'hardware': hardware.toJson(),
    };
  }

  AppSettings copyWith({
    int? schemaVersion,
    UiSettings? ui,
    HardwareSettings? hardware,
  }) {
    return AppSettings(
      schemaVersion: schemaVersion ?? this.schemaVersion,
      ui: ui ?? this.ui,
      hardware: hardware ?? this.hardware,
    );
  }

  @override
  bool operator ==(Object other) {
    return other is AppSettings &&
        other.schemaVersion == schemaVersion &&
        other.ui == ui &&
        other.hardware == hardware;
  }

  @override
  int get hashCode => Object.hash(schemaVersion, ui, hardware);
}

class UiSettings {
  final int selectedNavigationIndex;
  final String? activeHardwareProfileId;

  const UiSettings({
    this.selectedNavigationIndex = 0,
    this.activeHardwareProfileId,
  });

  factory UiSettings.fromJson(Map<String, dynamic> json, List<String>? issues) {
    return UiSettings(
      selectedNavigationIndex: _readRangedInt(
        json,
        'selectedNavigationIndex',
        min: 0,
        max: 1000,
        defaultValue: 0,
        issues: issues,
        path: 'ui.selectedNavigationIndex',
      ),
      activeHardwareProfileId: _readOptionalString(
        json,
        'activeHardwareProfileId',
        issues: issues,
        path: 'ui.activeHardwareProfileId',
      ),
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'selectedNavigationIndex': selectedNavigationIndex,
      if (activeHardwareProfileId != null)
        'activeHardwareProfileId': activeHardwareProfileId,
    };
  }

  UiSettings copyWith({
    int? selectedNavigationIndex,
    Object? activeHardwareProfileId = _unset,
  }) {
    return UiSettings(
      selectedNavigationIndex:
          selectedNavigationIndex ?? this.selectedNavigationIndex,
      activeHardwareProfileId: identical(activeHardwareProfileId, _unset)
          ? this.activeHardwareProfileId
          : activeHardwareProfileId as String?,
    );
  }

  @override
  bool operator ==(Object other) {
    return other is UiSettings &&
        other.selectedNavigationIndex == selectedNavigationIndex &&
        other.activeHardwareProfileId == activeHardwareProfileId;
  }

  @override
  int get hashCode =>
      Object.hash(selectedNavigationIndex, activeHardwareProfileId);
}

class HardwareSettings {
  final int? performanceMode;
  final TdpDesiredSettings? tdp;
  final String? cpuBoostMode;
  final int? batteryChargeLimit;
  final String? gpuMode;
  final FanDesiredSettings? fan;

  const HardwareSettings({
    this.performanceMode,
    this.tdp,
    this.cpuBoostMode,
    this.batteryChargeLimit,
    this.gpuMode,
    this.fan,
  });

  factory HardwareSettings.fromJson(
    Map<String, dynamic> json,
    List<String>? issues,
  ) {
    return HardwareSettings(
      performanceMode: _readOptionalRangedInt(
        json,
        'performanceMode',
        min: 0,
        max: 2,
        issues: issues,
        path: 'hardware.performanceMode',
      ),
      tdp: _readOptionalObject(
        json,
        'tdp',
        issues: issues,
        path: 'hardware.tdp',
        parse: (value) => TdpDesiredSettings.fromJson(value, issues),
      ),
      cpuBoostMode: _readOptionalString(
        json,
        'cpuBoostMode',
        issues: issues,
        path: 'hardware.cpuBoostMode',
      ),
      batteryChargeLimit: _readOptionalRangedInt(
        json,
        'batteryChargeLimit',
        min: 0,
        max: 100,
        issues: issues,
        path: 'hardware.batteryChargeLimit',
      ),
      gpuMode: _readOptionalString(
        json,
        'gpuMode',
        issues: issues,
        path: 'hardware.gpuMode',
      ),
      fan: _readOptionalObject(
        json,
        'fan',
        issues: issues,
        path: 'hardware.fan',
        parse: (value) => FanDesiredSettings.fromJson(value, issues),
      ),
    );
  }

  Map<String, dynamic> toJson() {
    return {
      if (performanceMode != null) 'performanceMode': performanceMode,
      if (tdp != null) 'tdp': tdp!.toJson(),
      if (cpuBoostMode != null) 'cpuBoostMode': cpuBoostMode,
      if (batteryChargeLimit != null) 'batteryChargeLimit': batteryChargeLimit,
      if (gpuMode != null) 'gpuMode': gpuMode,
      if (fan != null) 'fan': fan!.toJson(),
    };
  }

  HardwareSettings copyWith({
    Object? performanceMode = _unset,
    Object? tdp = _unset,
    Object? cpuBoostMode = _unset,
    Object? batteryChargeLimit = _unset,
    Object? gpuMode = _unset,
    Object? fan = _unset,
  }) {
    return HardwareSettings(
      performanceMode: identical(performanceMode, _unset)
          ? this.performanceMode
          : performanceMode as int?,
      tdp: identical(tdp, _unset) ? this.tdp : tdp as TdpDesiredSettings?,
      cpuBoostMode: identical(cpuBoostMode, _unset)
          ? this.cpuBoostMode
          : cpuBoostMode as String?,
      batteryChargeLimit: identical(batteryChargeLimit, _unset)
          ? this.batteryChargeLimit
          : batteryChargeLimit as int?,
      gpuMode: identical(gpuMode, _unset) ? this.gpuMode : gpuMode as String?,
      fan: identical(fan, _unset) ? this.fan : fan as FanDesiredSettings?,
    );
  }

  @override
  bool operator ==(Object other) {
    return other is HardwareSettings &&
        other.performanceMode == performanceMode &&
        other.tdp == tdp &&
        other.cpuBoostMode == cpuBoostMode &&
        other.batteryChargeLimit == batteryChargeLimit &&
        other.gpuMode == gpuMode &&
        other.fan == fan;
  }

  @override
  int get hashCode {
    return Object.hash(
      performanceMode,
      tdp,
      cpuBoostMode,
      batteryChargeLimit,
      gpuMode,
      fan,
    );
  }
}

class TdpDesiredSettings {
  final bool powerLimitsEnabled;
  final int sustainedW;
  final int boostW;
  final bool tempLimitEnabled;
  final int tempLimitC;

  const TdpDesiredSettings({
    required this.powerLimitsEnabled,
    required this.sustainedW,
    required this.boostW,
    required this.tempLimitEnabled,
    required this.tempLimitC,
  });

  factory TdpDesiredSettings.fromJson(
    Map<String, dynamic> json,
    List<String>? issues,
  ) {
    return TdpDesiredSettings(
      powerLimitsEnabled: _readBool(
        json,
        'powerLimitsEnabled',
        defaultValue: false,
        issues: issues,
        path: 'hardware.tdp.powerLimitsEnabled',
      ),
      sustainedW: _readRangedInt(
        json,
        'sustainedW',
        min: 0,
        max: 150,
        defaultValue: 0,
        issues: issues,
        path: 'hardware.tdp.sustainedW',
      ),
      boostW: _readRangedInt(
        json,
        'boostW',
        min: 0,
        max: 150,
        defaultValue: 0,
        issues: issues,
        path: 'hardware.tdp.boostW',
      ),
      tempLimitEnabled: _readBool(
        json,
        'tempLimitEnabled',
        defaultValue: false,
        issues: issues,
        path: 'hardware.tdp.tempLimitEnabled',
      ),
      tempLimitC: _readRangedInt(
        json,
        'tempLimitC',
        min: 0,
        max: 105,
        defaultValue: 0,
        issues: issues,
        path: 'hardware.tdp.tempLimitC',
      ),
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'powerLimitsEnabled': powerLimitsEnabled,
      'sustainedW': sustainedW,
      'boostW': boostW,
      'tempLimitEnabled': tempLimitEnabled,
      'tempLimitC': tempLimitC,
    };
  }

  @override
  bool operator ==(Object other) {
    return other is TdpDesiredSettings &&
        other.powerLimitsEnabled == powerLimitsEnabled &&
        other.sustainedW == sustainedW &&
        other.boostW == boostW &&
        other.tempLimitEnabled == tempLimitEnabled &&
        other.tempLimitC == tempLimitC;
  }

  @override
  int get hashCode {
    return Object.hash(
      powerLimitsEnabled,
      sustainedW,
      boostW,
      tempLimitEnabled,
      tempLimitC,
    );
  }
}

class FanDesiredSettings {
  final String? selectedProfile;
  final String? selectedFan;
  final Map<String, String> controlModes;
  final Map<String, dynamic>? dynamicEngineConfig;

  const FanDesiredSettings({
    this.selectedProfile,
    this.selectedFan,
    this.controlModes = const {},
    this.dynamicEngineConfig,
  });

  factory FanDesiredSettings.fromJson(
    Map<String, dynamic> json,
    List<String>? issues,
  ) {
    return FanDesiredSettings(
      selectedProfile: _readOptionalString(
        json,
        'selectedProfile',
        issues: issues,
        path: 'hardware.fan.selectedProfile',
      ),
      selectedFan: _readOptionalString(
        json,
        'selectedFan',
        issues: issues,
        path: 'hardware.fan.selectedFan',
      ),
      controlModes: _readStringMap(
        json,
        'controlModes',
        issues: issues,
        path: 'hardware.fan.controlModes',
      ),
      dynamicEngineConfig: _readOptionalMap(
        json,
        'dynamicEngineConfig',
        issues: issues,
        path: 'hardware.fan.dynamicEngineConfig',
      ),
    );
  }

  Map<String, dynamic> toJson() {
    return {
      if (selectedProfile != null) 'selectedProfile': selectedProfile,
      if (selectedFan != null) 'selectedFan': selectedFan,
      if (controlModes.isNotEmpty) 'controlModes': controlModes,
      if (dynamicEngineConfig != null)
        'dynamicEngineConfig': dynamicEngineConfig,
    };
  }

  FanDesiredSettings copyWith({
    Object? selectedProfile = _unset,
    Object? selectedFan = _unset,
    Map<String, String>? controlModes,
    Object? dynamicEngineConfig = _unset,
  }) {
    return FanDesiredSettings(
      selectedProfile: identical(selectedProfile, _unset)
          ? this.selectedProfile
          : selectedProfile as String?,
      selectedFan: identical(selectedFan, _unset)
          ? this.selectedFan
          : selectedFan as String?,
      controlModes: controlModes ?? this.controlModes,
      dynamicEngineConfig: identical(dynamicEngineConfig, _unset)
          ? this.dynamicEngineConfig
          : dynamicEngineConfig as Map<String, dynamic>?,
    );
  }

  @override
  bool operator ==(Object other) {
    return other is FanDesiredSettings &&
        other.selectedProfile == selectedProfile &&
        other.selectedFan == selectedFan &&
        _mapEquals(other.controlModes, controlModes) &&
        _dynamicMapEquals(other.dynamicEngineConfig, dynamicEngineConfig);
  }

  @override
  int get hashCode {
    return Object.hash(
      selectedProfile,
      selectedFan,
      Object.hashAll(controlModes.entries),
      dynamicEngineConfig?.toString(),
    );
  }
}

const _unset = Object();

Map<String, dynamic> _readMap(
  Map<String, dynamic> json,
  String key, {
  List<String>? issues,
}) {
  final value = json[key];
  if (value == null) return const {};
  if (value is Map<String, dynamic>) return value;
  issues?.add('$key invalidType actual=${value.runtimeType}');
  return const {};
}

Map<String, dynamic>? _readOptionalMap(
  Map<String, dynamic> json,
  String key, {
  required String path,
  List<String>? issues,
}) {
  final value = json[key];
  if (value == null) return null;
  if (value is Map<String, dynamic>) return value;
  issues?.add('$path invalidType actual=${value.runtimeType}');
  return null;
}

T? _readOptionalObject<T>(
  Map<String, dynamic> json,
  String key, {
  required String path,
  required T Function(Map<String, dynamic> value) parse,
  List<String>? issues,
}) {
  final value = json[key];
  if (value == null) return null;
  if (value is Map<String, dynamic>) return parse(value);
  issues?.add('$path invalidType actual=${value.runtimeType}');
  return null;
}

int _readInt(
  Map<String, dynamic> json,
  String key, {
  required int defaultValue,
  List<String>? issues,
}) {
  final value = json[key];
  if (value == null) return defaultValue;
  if (value is int) return value;
  issues?.add('$key invalidType actual=${value.runtimeType}');
  return defaultValue;
}

int _readRangedInt(
  Map<String, dynamic> json,
  String key, {
  required int min,
  required int max,
  required int defaultValue,
  required String path,
  List<String>? issues,
}) {
  final value = json[key];
  if (value == null) return defaultValue;
  if (value is! int) {
    issues?.add('$path invalidType actual=${value.runtimeType}');
    return defaultValue;
  }
  if (value < min || value > max) {
    issues?.add('$path outOfRange actual=$value expected=$min..$max');
    return defaultValue;
  }
  return value;
}

int? _readOptionalRangedInt(
  Map<String, dynamic> json,
  String key, {
  required int min,
  required int max,
  required String path,
  List<String>? issues,
}) {
  final value = json[key];
  if (value == null) return null;
  if (value is! int) {
    issues?.add('$path invalidType actual=${value.runtimeType}');
    return null;
  }
  if (value < min || value > max) {
    issues?.add('$path outOfRange actual=$value expected=$min..$max');
    return null;
  }
  return value;
}

bool _readBool(
  Map<String, dynamic> json,
  String key, {
  required bool defaultValue,
  required String path,
  List<String>? issues,
}) {
  final value = json[key];
  if (value == null) return defaultValue;
  if (value is bool) return value;
  issues?.add('$path invalidType actual=${value.runtimeType}');
  return defaultValue;
}

String? _readOptionalString(
  Map<String, dynamic> json,
  String key, {
  required String path,
  List<String>? issues,
}) {
  final value = json[key];
  if (value == null) return null;
  if (value is String && value.isNotEmpty) return value;
  issues?.add('$path invalidValue');
  return null;
}

Map<String, String> _readStringMap(
  Map<String, dynamic> json,
  String key, {
  required String path,
  List<String>? issues,
}) {
  final value = json[key];
  if (value == null) return const {};
  if (value is! Map<String, dynamic>) {
    issues?.add('$path invalidType actual=${value.runtimeType}');
    return const {};
  }

  final result = <String, String>{};
  for (final entry in value.entries) {
    if (entry.value is String) {
      result[entry.key] = entry.value as String;
    } else {
      issues?.add('$path.${entry.key} invalidType');
    }
  }
  return result;
}

bool _mapEquals(Map<String, String> left, Map<String, String> right) {
  if (left.length != right.length) return false;
  for (final entry in left.entries) {
    if (right[entry.key] != entry.value) return false;
  }
  return true;
}

bool _dynamicMapEquals(
  Map<String, dynamic>? left,
  Map<String, dynamic>? right,
) {
  if (identical(left, right)) return true;
  if (left == null || right == null) return false;
  if (left.length != right.length) return false;
  for (final entry in left.entries) {
    if (right[entry.key] != entry.value) return false;
  }
  return true;
}
