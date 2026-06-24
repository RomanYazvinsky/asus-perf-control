import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../blocs/tdp/tdp_bloc.dart';
import '../blocs/tdp/tdp_event.dart';
import '../blocs/tdp/tdp_state.dart';
import '../widgets/copyable_alert.dart';
import '../widgets/tdp/tdp_controls.dart';
import '../widgets/tdp/tdp_unsupported_card.dart';

class TdpScreen extends StatefulWidget {
  final TdpBloc? testBloc;

  const TdpScreen({super.key, this.testBloc});

  @override
  State<TdpScreen> createState() => _TdpScreenState();
}

class _TdpScreenState extends State<TdpScreen> {
  late final TdpBloc _tdpBloc;
  double _sustainedW = 0;
  double _boostW = 0;
  double _tempLimitC = 90;
  double _minW = 5;
  double _maxW = 54;
  double _minBoostW = 5;
  double _maxBoostW = 54;
  bool _hasLoadedData = false;
  bool _powerLimitsEnabled = false;
  bool _tempLimitEnabled = false;

  @override
  void initState() {
    super.initState();
    _tdpBloc = widget.testBloc ?? (TdpBloc()..add(LoadTdpConfig()));
  }

  @override
  void dispose() {
    _tdpBloc.close();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('CPU Power & Temperature Limits')),
      body: BlocProvider<TdpBloc>.value(
        value: _tdpBloc,
        child: BlocListener<TdpBloc, TdpState>(
          listener: (context, state) {
            if (state is TdpError) {
              CopyableAlert.showError(
                context,
                title: 'TDP Error',
                message: _hasLoadedData
                    ? 'Failed to apply TDP settings'
                    : 'Failed to load TDP config',
                errorDetails: state.message,
              );
            } else if (state is TdpLoaded) {
              setState(() {
                // Use actual system values when non-zero; fall back to safe defaults only when
                // hardware reports 0 (TDP never configured — firmware resets on reboot).
                _minW = state.minW.toDouble();
                _maxW = state.maxW.toDouble();
                _minBoostW = state.minBoostW.toDouble();
                _maxBoostW = state.maxBoostW.toDouble();
                _sustainedW = state.sustainedW > 0
                    ? state.sustainedW.toDouble().clamp(_minW, _maxW)
                    : _minW;
                _boostW = state.boostW > 0
                    ? state.boostW.toDouble().clamp(_minBoostW, _maxBoostW)
                    : _minBoostW;
                _tempLimitC = state.tempLimitC > 0
                    ? state.tempLimitC.toDouble().clamp(60.0, 100.0)
                    : _tempLimitC;
                _hasLoadedData = true;
                _powerLimitsEnabled = state.powerLimitsEnabled;
                _tempLimitEnabled = state.tempLimitEnabled;
              });
            }
          },
          child: BlocBuilder<TdpBloc, TdpState>(
            builder: (context, state) {
              if (state is TdpUnsupported) {
                return const TdpUnsupportedCard();
              }
              if (state is TdpLoading ||
                  state is TdpInitial ||
                  state is TdpError ||
                  !_hasLoadedData) {
                return const Center(child: CircularProgressIndicator());
              }
              return TdpControls(
                state: state,
                sustainedW: _sustainedW,
                boostW: _boostW,
                tempLimitC: _tempLimitC,
                minW: _minW,
                maxW: _maxW,
                minBoostW: _minBoostW,
                maxBoostW: _maxBoostW,
                powerLimitsEnabled: _powerLimitsEnabled,
                tempLimitEnabled: _tempLimitEnabled,
                onPowerLimitsEnabledChanged: (v) {
                  setState(() => _powerLimitsEnabled = v);
                  if (!v) {
                    _tdpBloc.add(ClearPowerLimitsRequested());
                  }
                },
                onTempLimitEnabledChanged: (v) =>
                    setState(() => _tempLimitEnabled = v),
                onSustainedChanged: (v) => setState(() => _sustainedW = v),
                onBoostChanged: (v) => setState(() => _boostW = v),
                onTempChanged: (v) => setState(() => _tempLimitC = v),
              );
            },
          ),
        ),
      ),
    );
  }
}
