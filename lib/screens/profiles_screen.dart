import 'package:flutter/material.dart';

import '../services/settings/app_settings_store.dart';
import '../services/settings/hardware_profile_store.dart';

class ProfilesScreen extends StatefulWidget {
  final AppSettingsStore settingsStore;
  final HardwareProfileStore profileStore;
  final VoidCallback onProfileApplied;

  const ProfilesScreen({
    super.key,
    required this.settingsStore,
    required this.profileStore,
    required this.onProfileApplied,
  });

  @override
  State<ProfilesScreen> createState() => _ProfilesScreenState();
}

class _ProfilesScreenState extends State<ProfilesScreen> {
  final TextEditingController _nameController = TextEditingController();
  var _profiles = const <HardwareProfile>[];
  String? _activeProfileId;
  bool _isAdding = false;
  bool _isLoading = true;

  @override
  void initState() {
    super.initState();
    _loadProfiles();
  }

  @override
  void dispose() {
    _nameController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Profiles')),
      body: Padding(
        padding: const EdgeInsets.all(24),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            Align(alignment: Alignment.centerLeft, child: _buildAddArea()),
            const SizedBox(height: 16),
            Expanded(
              child: _isLoading
                  ? const Center(child: CircularProgressIndicator())
                  : _buildProfilesTable(),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildAddArea() {
    if (!_isAdding) {
      return FilledButton.icon(
        onPressed: () => setState(() => _isAdding = true),
        icon: const Icon(Icons.add),
        label: const Text('Add'),
      );
    }

    return ConstrainedBox(
      constraints: const BoxConstraints(maxWidth: 560),
      child: Row(
        children: [
          Expanded(
            child: TextField(
              controller: _nameController,
              autofocus: true,
              decoration: const InputDecoration(
                labelText: 'Profile name',
                border: OutlineInputBorder(),
              ),
              onSubmitted: (_) => _saveProfile(),
            ),
          ),
          const SizedBox(width: 8),
          FilledButton(onPressed: _saveProfile, child: const Text('Save')),
          const SizedBox(width: 8),
          TextButton(onPressed: _cancelAdd, child: const Text('Cancel')),
        ],
      ),
    );
  }

  Widget _buildProfilesTable() {
    if (_profiles.isEmpty) {
      return const Align(
        alignment: Alignment.topLeft,
        child: Text('No profiles'),
      );
    }

    return ListView(
      children: [
        const _ProfileTableHeader(),
        for (final profile in _profiles) _buildProfileRow(profile),
      ],
    );
  }

  Widget _buildProfileRow(HardwareProfile profile) {
    final isActive = profile.id == _activeProfileId;
    return Container(
      key: isActive ? Key('active_profile_row_${profile.id}') : null,
      decoration: BoxDecoration(
        color: isActive ? Colors.green.withValues(alpha: 0.18) : null,
        border: Border(
          bottom: BorderSide(color: Theme.of(context).dividerColor),
        ),
      ),
      child: Padding(
        padding: const EdgeInsets.symmetric(vertical: 8, horizontal: 12),
        child: Row(
          children: [
            Expanded(child: Text(profile.name)),
            TextButton(
              key: Key('apply_profile_${profile.id}'),
              onPressed: isActive ? null : () => _applyProfile(profile),
              child: const Text('Apply'),
            ),
            const SizedBox(width: 8),
            TextButton(
              key: Key('delete_profile_${profile.id}'),
              onPressed: isActive ? null : () => _deleteProfile(profile),
              child: const Text('Delete'),
            ),
          ],
        ),
      ),
    );
  }

  Future<void> _loadProfiles() async {
    final settings = await widget.settingsStore.load();
    final profiles = await widget.profileStore.list();
    if (!mounted) return;
    setState(() {
      _activeProfileId = settings.ui.activeHardwareProfileId;
      _profiles = profiles;
      _isLoading = false;
    });
  }

  Future<void> _saveProfile() async {
    final name = _nameController.text.trim();
    if (name.isEmpty) return;

    final settings = await widget.settingsStore.load();
    final profile = await widget.profileStore.create(
      name: name,
      hardware: settings.hardware,
    );
    await _setActiveProfile(profile.id);
    _cancelAdd();
    widget.onProfileApplied();
    await _loadProfiles();
  }

  Future<void> _applyProfile(HardwareProfile profile) async {
    await _setActiveProfile(profile.id);
    widget.onProfileApplied();
    await _loadProfiles();
  }

  Future<void> _deleteProfile(HardwareProfile profile) async {
    if (profile.id == _activeProfileId) return;
    await widget.profileStore.delete(profile.id);
    await _loadProfiles();
  }

  Future<void> _setActiveProfile(String id) async {
    await widget.settingsStore.update((settings) {
      return settings.copyWith(
        ui: settings.ui.copyWith(activeHardwareProfileId: id),
      );
    });
  }

  void _cancelAdd() {
    if (!mounted) return;
    setState(() {
      _isAdding = false;
      _nameController.clear();
    });
  }
}

class _ProfileTableHeader extends StatelessWidget {
  const _ProfileTableHeader();

  @override
  Widget build(BuildContext context) {
    final style = Theme.of(context).textTheme.labelLarge;
    return Container(
      decoration: BoxDecoration(
        border: Border(
          bottom: BorderSide(color: Theme.of(context).dividerColor),
        ),
      ),
      padding: const EdgeInsets.symmetric(vertical: 8, horizontal: 12),
      child: Row(
        children: [
          Expanded(child: Text('Profile', style: style)),
          SizedBox(width: 72, child: Text('Apply', style: style)),
          SizedBox(width: 72, child: Text('Delete', style: style)),
        ],
      ),
    );
  }
}
