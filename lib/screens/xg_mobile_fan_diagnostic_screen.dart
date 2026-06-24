import 'package:flutter/material.dart';

import '../services/xg_mobile_fan_diagnostic_service.dart';

class XgMobileFanDiagnosticScreen extends StatefulWidget {
  final XgMobileFanDiagnosticService service;
  final XgFanDiagnosticStore store;

  XgMobileFanDiagnosticScreen({
    super.key,
    XgMobileFanDiagnosticService? service,
    XgFanDiagnosticStore? store,
  }) : service = service ?? _DefaultServiceFactory().create(),
       store = store ?? FileXgFanDiagnosticStore();

  @override
  State<XgMobileFanDiagnosticScreen> createState() =>
      _XgMobileFanDiagnosticScreenState();
}

class _DefaultServiceFactory {
  const _DefaultServiceFactory();

  XgMobileFanDiagnosticService create() => XgMobileFanDiagnosticService();
}

class _XgMobileFanDiagnosticScreenState
    extends State<XgMobileFanDiagnosticScreen> {
  XgFanDiagnosticRun? _currentRun;
  XgFanDiagnosticRun? _changedRun;
  FanHypothesisResult? _hypothesisResult;
  String? _error;
  String? _runningLabel;
  bool _loadingSavedSession = true;

  bool get _isRunning => _runningLabel != null;

  @override
  void initState() {
    super.initState();
    _loadSavedSession();
  }

  Future<void> _loadSavedSession() async {
    try {
      final session = await widget.store.load();
      if (!mounted) {
        return;
      }
      setState(() {
        _currentRun = session.currentRun;
        _changedRun = session.changedRun;
        _loadingSavedSession = false;
      });
    } catch (error) {
      if (!mounted) {
        return;
      }
      setState(() {
        _error = 'Failed to load saved diagnostic session: $error';
        _loadingSavedSession = false;
      });
    }
  }

  Future<void> _runSweep({required bool changedState}) async {
    final label = changedState ? 'Changed XG Mobile State' : 'Current State';
    setState(() {
      _runningLabel = label;
      _error = null;
    });

    try {
      final run = await widget.service.runSweep(label: label);
      if (!mounted) {
        return;
      }
      setState(() {
        if (changedState) {
          _changedRun = run;
        } else {
          _currentRun = run;
        }
      });
      await widget.store.save(
        XgFanDiagnosticSession(
          currentRun: _currentRun,
          changedRun: _changedRun,
        ),
      );
    } catch (error) {
      if (!mounted) {
        return;
      }
      setState(() => _error = error.toString());
    } finally {
      if (mounted) {
        setState(() => _runningLabel = null);
      }
    }
  }

  Future<void> _runHypothesis(
    String label,
    Future<FanHypothesisResult> Function() action,
  ) async {
    setState(() {
      _runningLabel = label;
      _error = null;
    });
    try {
      final result = await action();
      if (!mounted) {
        return;
      }
      setState(() => _hypothesisResult = result);
    } catch (error) {
      if (!mounted) {
        return;
      }
      setState(() => _error = error.toString());
    } finally {
      if (mounted) {
        setState(() => _runningLabel = null);
      }
    }
  }

  XgFanDiagnosis? get _diagnosis {
    final current = _currentRun;
    final changed = _changedRun;
    if (current != null && changed != null) {
      return XgFanDiagnosis.compare(current, changed);
    }
    if (changed != null) {
      return XgFanDiagnosis.single(changed);
    }
    if (current != null) {
      return XgFanDiagnosis.single(current);
    }
    return null;
  }

  @override
  Widget build(BuildContext context) {
    final diagnosis = _diagnosis;
    return Scaffold(
      appBar: AppBar(title: const Text('XG Mobile Diagnostic')),
      body: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          _SafetyPanel(
            isRunning: _isRunning,
            runningLabel: _runningLabel,
            loadingSavedSession: _loadingSavedSession,
          ),
          const SizedBox(height: 12),
          if (_error != null) ...[
            _ErrorPanel(message: _error!),
            const SizedBox(height: 12),
          ],
          Row(
            children: [
              Expanded(
                child: _RunSlot(
                  title: 'Current State',
                  run: _currentRun,
                  isRunning: _runningLabel == 'Current State',
                  onRun: _isRunning
                      ? null
                      : () => _runSweep(changedState: false),
                  buttonText: 'Run Current State Sweep',
                ),
              ),
              const SizedBox(width: 12),
              Expanded(
                child: _RunSlot(
                  title: 'Changed XG Mobile State',
                  run: _changedRun,
                  isRunning: _runningLabel == 'Changed XG Mobile State',
                  onRun: _isRunning
                      ? null
                      : () => _runSweep(changedState: true),
                  buttonText: 'Run Changed State Sweep',
                ),
              ),
            ],
          ),
          const SizedBox(height: 12),
          _HypothesisPanel(
            isRunning: _isRunning,
            result: _hypothesisResult,
            onCpuGpuCoupling: () => _runHypothesis(
              'CPU/GPU Coupling',
              widget.service.runCpuGpuCouplingTest,
            ),
            onProfileBounds: () => _runHypothesis(
              'Profile Bounds',
              widget.service.runProfileBoundsTest,
            ),
            onCurveValidation: () => _runHypothesis(
              'Curve Rule Discovery',
              widget.service.runCurveValidationTest,
            ),
          ),
          const SizedBox(height: 12),
          if (diagnosis != null) _DiagnosisPanel(diagnosis: diagnosis),
        ],
      ),
    );
  }
}

class _HypothesisPanel extends StatelessWidget {
  final bool isRunning;
  final FanHypothesisResult? result;
  final VoidCallback onCpuGpuCoupling;
  final VoidCallback onProfileBounds;
  final VoidCallback onCurveValidation;

  const _HypothesisPanel({
    required this.isRunning,
    required this.result,
    required this.onCpuGpuCoupling,
    required this.onProfileBounds,
    required this.onCurveValidation,
  });

  @override
  Widget build(BuildContext context) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(12),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              'Fan Hypotheses',
              style: Theme.of(context).textTheme.titleMedium,
            ),
            const SizedBox(height: 8),
            Wrap(
              spacing: 8,
              runSpacing: 8,
              children: [
                FilledButton.icon(
                  onPressed: isRunning ? null : onCpuGpuCoupling,
                  icon: const Icon(Icons.compare_arrows),
                  label: const Text('Run CPU/GPU Coupling'),
                ),
                FilledButton.icon(
                  onPressed: isRunning ? null : onProfileBounds,
                  icon: const Icon(Icons.speed),
                  label: const Text('Run Profile Bounds'),
                ),
                FilledButton.icon(
                  onPressed: isRunning ? null : onCurveValidation,
                  icon: const Icon(Icons.rule),
                  label: const Text('Run Curve Rule Discovery'),
                ),
              ],
            ),
            if (result != null) ...[
              const SizedBox(height: 12),
              Text(result!.name, style: Theme.of(context).textTheme.titleSmall),
              const SizedBox(height: 4),
              Text(_hypothesisTitle(result!.diagnosis.kind)),
              const SizedBox(height: 4),
              Text(result!.diagnosis.summary),
              const SizedBox(height: 4),
              Text(result!.diagnosis.nextAction),
              const SizedBox(height: 8),
              for (final step in result!.steps)
                Text(_formatHypothesisStep(step)),
            ],
          ],
        ),
      ),
    );
  }
}

String _formatInt(int? value) => value == null ? 'n/a' : '$value';

String _formatHypothesisStep(FanHypothesisStep step) {
  if (step.defaultPwm != null ||
      step.readbackPwm != null ||
      step.cpuRpmBefore != null ||
      step.cpuRpmAfter != null ||
      step.cpuRpmDelta != null) {
    return '${step.label}: temp ${_formatInt(step.currentTempC)}C, '
        'PWM default=${_formatInt(step.defaultPwm)} '
        'requested=${step.requestedPwm} '
        'readback=${_formatInt(step.readbackPwm)}, '
        'CPU RPM ${_formatInt(step.cpuRpmBefore)}->${_formatInt(step.cpuRpmAfter)} '
        'delta=${_formatSignedInt(step.cpuRpmDelta)}';
  }
  return '${step.label}: ${step.requestedFan} PWM ${step.requestedPwm}, '
      'CPU RPM ${_formatInt(step.cpuLastRpm)}';
}

String _formatSignedInt(int? value) {
  if (value == null) {
    return 'n/a';
  }
  return value > 0 ? '+$value' : '$value';
}

class _SafetyPanel extends StatelessWidget {
  final bool isRunning;
  final String? runningLabel;
  final bool loadingSavedSession;

  const _SafetyPanel({
    required this.isRunning,
    required this.runningLabel,
    required this.loadingSavedSession,
  });

  @override
  Widget build(BuildContext context) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(12),
        child: Row(
          children: [
            Icon(
              isRunning ? Icons.sync : Icons.info_outline,
              color: isRunning ? Colors.orange : null,
            ),
            const SizedBox(width: 12),
            Expanded(
              child: Text(
                loadingSavedSession
                    ? 'Loading saved diagnostic session.'
                    : isRunning
                    ? 'Running $runningLabel. The app will restore balanced defaults when the sweep finishes.'
                    : 'Run the current state, reboot after changing XG Mobile state, then return here. The first run is saved for comparison after restart.',
              ),
            ),
          ],
        ),
      ),
    );
  }
}

class _ErrorPanel extends StatelessWidget {
  final String message;

  const _ErrorPanel({required this.message});

  @override
  Widget build(BuildContext context) {
    return Card(
      color: Theme.of(context).colorScheme.errorContainer,
      child: Padding(padding: const EdgeInsets.all(12), child: Text(message)),
    );
  }
}

class _RunSlot extends StatelessWidget {
  final String title;
  final XgFanDiagnosticRun? run;
  final bool isRunning;
  final VoidCallback? onRun;
  final String buttonText;

  const _RunSlot({
    required this.title,
    required this.run,
    required this.isRunning,
    required this.onRun,
    required this.buttonText,
  });

  @override
  Widget build(BuildContext context) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(12),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(title, style: Theme.of(context).textTheme.titleMedium),
            const SizedBox(height: 8),
            FilledButton.icon(
              onPressed: onRun,
              icon: isRunning
                  ? const SizedBox.square(
                      dimension: 18,
                      child: CircularProgressIndicator(strokeWidth: 2),
                    )
                  : const Icon(Icons.science_outlined),
              label: Text(buttonText),
            ),
            const SizedBox(height: 12),
            if (run == null)
              const Text('No run captured')
            else
              _RunSummary(run: run!),
          ],
        ),
      ),
    );
  }
}

class _RunSummary extends StatelessWidget {
  final XgFanDiagnosticRun run;

  const _RunSummary({required this.run});

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text('Baseline fan2: ${_rpm(run.baselineSnapshot.fan2Rpm)}'),
        const SizedBox(height: 8),
        Table(
          columnWidths: const {
            0: FixedColumnWidth(52),
            1: FixedColumnWidth(74),
            2: FixedColumnWidth(74),
            3: FixedColumnWidth(74),
            4: FixedColumnWidth(88),
          },
          children: [
            const TableRow(
              children: [
                _HeaderCell('PWM'),
                _HeaderCell('Min'),
                _HeaderCell('Max'),
                _HeaderCell('Last'),
                _HeaderCell('Result'),
              ],
            ),
            for (final step in run.steps)
              TableRow(
                children: [
                  _BodyCell('${step.requestedPwm}'),
                  _BodyCell(_rpm(step.fan2MinRpm)),
                  _BodyCell(_rpm(step.fan2MaxRpm)),
                  _BodyCell(_rpm(step.fan2LastRpm)),
                  _BodyCell(_classificationLabel(step.classification)),
                ],
              ),
          ],
        ),
      ],
    );
  }
}

class _DiagnosisPanel extends StatelessWidget {
  final XgFanDiagnosis diagnosis;

  const _DiagnosisPanel({required this.diagnosis});

  @override
  Widget build(BuildContext context) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(12),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              _diagnosisTitle(diagnosis.kind),
              style: Theme.of(context).textTheme.titleMedium,
            ),
            const SizedBox(height: 8),
            Text(diagnosis.summary),
            const SizedBox(height: 6),
            Text(diagnosis.nextAction),
          ],
        ),
      ),
    );
  }
}

class _HeaderCell extends StatelessWidget {
  final String text;

  const _HeaderCell(this.text);

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4),
      child: Text(text, style: const TextStyle(fontWeight: FontWeight.bold)),
    );
  }
}

class _BodyCell extends StatelessWidget {
  final String text;

  const _BodyCell(this.text);

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4),
      child: Text(text),
    );
  }
}

String _rpm(int? value) => value == null ? 'n/a' : '$value';

String _classificationLabel(XgFanStepClassification classification) {
  switch (classification) {
    case XgFanStepClassification.responded:
      return 'Responded';
    case XgFanStepClassification.acceptedNoResponse:
      return 'No response';
    case XgFanStepClassification.inconclusive:
      return 'Unknown';
  }
}

String _diagnosisTitle(XgFanDiagnosisKind kind) {
  switch (kind) {
    case XgFanDiagnosisKind.responded:
      return 'Fan responded';
    case XgFanDiagnosisKind.acceptedNoResponse:
      return 'Accepted, no RPM response';
    case XgFanDiagnosisKind.routeChanged:
      return 'XG Mobile changed fan routing';
    case XgFanDiagnosisKind.gpuDependsOnXgMobile:
      return 'GPU fan depends on XG Mobile state';
    case XgFanDiagnosisKind.newHwmonLikely:
      return 'New hwmon path likely';
    case XgFanDiagnosisKind.inconclusive:
      return 'Inconclusive';
  }
}

String _hypothesisTitle(FanHypothesisKind kind) {
  switch (kind) {
    case FanHypothesisKind.gpuDependsOnCpuFan:
      return 'GPU fan follows CPU fan policy';
    case FanHypothesisKind.profileFloorOrCap:
      return 'Profile floor or cap detected';
    case FanHypothesisKind.downwardApplied:
      return 'Below-default curve applied';
    case FanHypothesisKind.downwardStoredButRuntimeClamped:
      return 'Below-default curve stored, runtime clamped';
    case FanHypothesisKind.downwardTransformedToDefault:
      return 'Below-default curve transformed to default';
    case FanHypothesisKind.upwardApplied:
      return 'Above-default curve applied';
    case FanHypothesisKind.curveNoRuntimeEffect:
      return 'Curve stored with no runtime effect';
    case FanHypothesisKind.curveFollowsRuntime:
      return 'Curve follows interpolated runtime expectation';
    case FanHypothesisKind.curveTransformed:
      return 'Curve transformed before use';
    case FanHypothesisKind.curveStoredButIgnored:
      return 'Curve stored but ignored at runtime';
    case FanHypothesisKind.curveInconclusive:
      return 'Runtime curve result inconclusive';
    case FanHypothesisKind.noEvidence:
      return 'No evidence in this run';
    case FanHypothesisKind.inconclusive:
      return 'Inconclusive';
  }
}
