import 'package:flutter/material.dart';
import '../../models/gpu_models.dart';

class GpuStatusCard extends StatelessWidget {
  final GpuStatus status;

  const GpuStatusCard({super.key, required this.status});

  @override
  Widget build(BuildContext context) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                const Icon(Icons.memory),
                const SizedBox(width: 8),
                Text(
                  'GPU Status',
                  style: Theme.of(context).textTheme.titleLarge,
                ),
              ],
            ),
            const Divider(),
            _buildStatusRow('Active Mode', _getModeName(status.activeMode)),
            _buildStatusRow('dGPU Enabled', status.dgpuEnabled ? 'Yes' : 'No'),
            if (status.temperature != null)
              _buildStatusRow('Temperature', '${status.temperature}°C'),
            if (status.utilization != null)
              _buildStatusRow('Utilization', '${status.utilization}%'),
            _buildStatusRow(
              'Power State',
              _getPowerStateName(status.powerState),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildStatusRow(String label, String value) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        children: [
          Text(label, style: const TextStyle(fontWeight: FontWeight.bold)),
          Text(value),
        ],
      ),
    );
  }

  String _getModeName(GpuMode mode) {
    switch (mode) {
      case GpuMode.integrated:
        return 'Integrated';
      case GpuMode.hybrid:
        return 'Hybrid';
      case GpuMode.ultimate:
        return 'Ultimate';
      case GpuMode.asusMuxDgpu:
        return 'MUX dGPU';
      case GpuMode.asusEgpu:
        return 'eGPU';
      case GpuMode.vfio:
        return 'VFIO';
    }
  }

  String _getPowerStateName(GpuPowerState state) {
    switch (state) {
      case GpuPowerState.active:
        return 'Active';
      case GpuPowerState.idle:
        return 'Idle';
      case GpuPowerState.suspended:
        return 'Suspended';
      case GpuPowerState.unknown:
        return 'Unknown';
    }
  }
}
