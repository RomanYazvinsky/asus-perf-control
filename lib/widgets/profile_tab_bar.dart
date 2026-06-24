import 'package:flutter/material.dart';

class ProfileTabBar extends StatefulWidget {
  final int selectedMode;
  final ValueChanged<int> onModeSelected;

  const ProfileTabBar({
    super.key,
    required this.selectedMode,
    required this.onModeSelected,
  });

  @override
  State<ProfileTabBar> createState() => _ProfileTabBarState();
}

class _ProfileTabBarState extends State<ProfileTabBar>
    with SingleTickerProviderStateMixin {
  late final TabController _controller;

  @override
  void initState() {
    super.initState();
    _controller = TabController(
      length: 3,
      vsync: this,
      initialIndex: _modeToIndex(widget.selectedMode),
    );
  }

  @override
  void didUpdateWidget(ProfileTabBar oldWidget) {
    super.didUpdateWidget(oldWidget);
    final nextIndex = _modeToIndex(widget.selectedMode);
    if (_controller.index != nextIndex) {
      _controller.index = nextIndex;
    }
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Material(
      color: Theme.of(context).colorScheme.surface,
      child: TabBar(
        key: const Key('profile_tab_bar'),
        controller: _controller,
        onTap: (index) => widget.onModeSelected(_indexToMode(index)),
        tabs: const [
          Tab(text: 'Silent', icon: Icon(Icons.volume_off)),
          Tab(text: 'Balanced', icon: Icon(Icons.balance)),
          Tab(text: 'Turbo', icon: Icon(Icons.speed)),
        ],
      ),
    );
  }

  int _modeToIndex(int mode) {
    return switch (mode) {
      0 => 0,
      1 => 1,
      2 => 2,
      _ => 1,
    };
  }

  int _indexToMode(int index) {
    return switch (index) {
      0 => 0,
      1 => 1,
      2 => 2,
      _ => 1,
    };
  }
}
