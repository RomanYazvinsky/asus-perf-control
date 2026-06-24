import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../blocs/package_updates/package_update_bloc.dart';
import '../blocs/package_updates/package_update_event.dart';
import '../blocs/package_updates/package_update_state.dart';
import '../services/package_update_service.dart';

class PackageUpdatesScreen extends StatefulWidget {
  const PackageUpdatesScreen({super.key, this.testBloc});

  final PackageUpdateBloc? testBloc;

  @override
  State<PackageUpdatesScreen> createState() => _PackageUpdatesScreenState();
}

class _PackageUpdatesScreenState extends State<PackageUpdatesScreen> {
  late final PackageUpdateBloc _bloc;

  @override
  void initState() {
    super.initState();
    _bloc =
        widget.testBloc ??
        (PackageUpdateBloc()..add(LoadPackageUpdateStatus()));
  }

  @override
  void dispose() {
    _bloc.close();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Package Updates')),
      body: BlocProvider<PackageUpdateBloc>.value(
        value: _bloc,
        child: BlocBuilder<PackageUpdateBloc, PackageUpdateState>(
          builder: (context, state) => ListView(
            padding: const EdgeInsets.all(20),
            children: [
              _StatusCard(state: state),
              const SizedBox(height: 16),
              _VersionCard(status: state.status),
              const SizedBox(height: 16),
              Wrap(
                spacing: 12,
                runSpacing: 12,
                children: [
                  OutlinedButton.icon(
                    onPressed: state.isChecking
                        ? null
                        : () => _bloc.add(CheckPackageUpdatesRequested()),
                    icon: const Icon(Icons.refresh),
                    label: const Text('Refresh'),
                  ),
                  ElevatedButton.icon(
                    onPressed: state.canStartUpdate
                        ? () => _bloc.add(StartPackageUpdateRequested())
                        : null,
                    icon: const Icon(Icons.system_update_alt),
                    label: const Text('Install Update'),
                  ),
                  if (state.isUpdateRunning)
                    OutlinedButton.icon(
                      onPressed: () =>
                          _bloc.add(CancelPackageUpdateRequested()),
                      icon: const Icon(Icons.cancel_outlined),
                      label: const Text('Cancel'),
                    ),
                ],
              ),
            ],
          ),
        ),
      ),
    );
  }
}

class _StatusCard extends StatelessWidget {
  const _StatusCard({required this.state});

  final PackageUpdateState state;

  @override
  Widget build(BuildContext context) {
    final status = state.status;
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Icon(_statusIcon(status?.state, state.daemonUnavailable)),
                const SizedBox(width: 12),
                Expanded(
                  child: Text(
                    state.summary,
                    style: Theme.of(context).textTheme.titleMedium,
                  ),
                ),
              ],
            ),
            if (state.isChecking) ...[
              const SizedBox(height: 12),
              const LinearProgressIndicator(),
            ],
            const SizedBox(height: 12),
            Text('Backend'),
            const SizedBox(height: 4),
            Text(
              status?.backend.wireName ?? state.backend?.wireName ?? 'unknown',
            ),
            if (status?.logTail.isNotEmpty == true) ...[
              const SizedBox(height: 12),
              ...status!.logTail.map((line) => Text(line)),
            ],
          ],
        ),
      ),
    );
  }
}

class _VersionCard extends StatelessWidget {
  const _VersionCard({required this.status});

  final PackageUpdateStatus? status;

  @override
  Widget build(BuildContext context) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text('Versions', style: Theme.of(context).textTheme.titleMedium),
            const SizedBox(height: 12),
            _InfoRow(
              label: 'Installed',
              value: status?.installedVersion ?? 'unknown',
            ),
            const SizedBox(height: 8),
            _InfoRow(
              label: 'Available',
              value: status?.availableVersion ?? 'none',
            ),
          ],
        ),
      ),
    );
  }
}

class _InfoRow extends StatelessWidget {
  const _InfoRow({required this.label, required this.value});

  final String label;
  final String value;

  @override
  Widget build(BuildContext context) {
    return Row(
      children: [
        SizedBox(width: 96, child: Text(label)),
        Expanded(child: Text(value)),
      ],
    );
  }
}

IconData _statusIcon(PackageUpdateRunState? state, bool daemonUnavailable) {
  if (daemonUnavailable) {
    return Icons.cloud_off_outlined;
  }
  switch (state) {
    case PackageUpdateRunState.available:
      return Icons.download_outlined;
    case PackageUpdateRunState.running:
      return Icons.sync;
    case PackageUpdateRunState.succeeded:
      return Icons.check_circle_outline;
    case PackageUpdateRunState.failed:
    case PackageUpdateRunState.unsupported:
      return Icons.error_outline;
    case PackageUpdateRunState.upToDate:
      return Icons.verified_outlined;
    case PackageUpdateRunState.idle:
    case PackageUpdateRunState.checking:
    case null:
      return Icons.info_outline;
  }
}
