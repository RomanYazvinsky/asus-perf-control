// Fan curve editor widget using fl_chart

import 'package:flutter/material.dart';
import 'package:fl_chart/fl_chart.dart';
import '../../services/asusd_fan_client.dart';
import '../../services/input/ui_input_command.dart';
import '../input/gamepad_input_scope.dart';

class FanCurveEditor extends StatefulWidget {
  final List<FanCurvePoint> curve;
  final ValueChanged<List<FanCurvePoint>>? onCurveChanged;
  final bool readOnly;
  final double? currentTemp;
  final double? currentSpeed;

  const FanCurveEditor({
    super.key,
    required this.curve,
    this.onCurveChanged,
    this.readOnly = false,
    this.currentTemp,
    this.currentSpeed,
  });

  @override
  State<FanCurveEditor> createState() => _FanCurveEditorState();
}

class _FanCurveEditorState extends State<FanCurveEditor> {
  List<FanCurvePoint> _points = [];
  int? _draggedPointIndex;
  int? _selectedPointIndex;
  final FocusNode _focusNode = FocusNode(debugLabel: 'FanCurveEditor');

  @override
  void initState() {
    super.initState();
    _points = List.from(widget.curve);
  }

  @override
  void dispose() {
    _focusNode.dispose();
    super.dispose();
  }

  @override
  void didUpdateWidget(FanCurveEditor oldWidget) {
    super.didUpdateWidget(oldWidget);
    // Only reset if not currently dragging and the curve actually changed
    if (_draggedPointIndex == null &&
        !_curvesEqual(oldWidget.curve, widget.curve)) {
      _points = List.from(widget.curve);
    }
  }

  bool _curvesEqual(List<FanCurvePoint> a, List<FanCurvePoint> b) {
    if (a.length != b.length) return false;
    for (int i = 0; i < a.length; i++) {
      if (a[i].tempC != b[i].tempC || a[i].speedPercent != b[i].speedPercent) {
        return false;
      }
    }
    return true;
  }

  @override
  Widget build(BuildContext context) {
    return Actions(
      actions: {
        UiInputIntent: _FanCurveInputAction(_handleUiInputCommand, context),
      },
      child: FocusableActionDetector(
        key: const ValueKey('fan-curve-editor-focus'),
        focusNode: _focusNode,
        onFocusChange: (focused) {
          if (focused && _selectedPointIndex == null && _points.isNotEmpty) {
            setState(() => _selectedPointIndex = 0);
          }
        },
        child: Card(
          child: Padding(
            padding: const EdgeInsets.all(16.0),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Row(
                  mainAxisAlignment: MainAxisAlignment.spaceBetween,
                  children: [
                    Text(
                      'Fan Curve (8 points)',
                      style: Theme.of(context).textTheme.titleLarge,
                    ),
                    if (!widget.readOnly)
                      ElevatedButton(
                        onPressed: () {
                          widget.onCurveChanged?.call(_points);
                        },
                        child: const Text('Apply'),
                      ),
                  ],
                ),
                const SizedBox(height: 16),
                SizedBox(
                  height: 300,
                  child: LayoutBuilder(
                    builder: (context, constraints) {
                      return GestureDetector(
                        onPanStart: !widget.readOnly
                            ? (details) {
                                RenderBox box =
                                    context.findRenderObject() as RenderBox;
                                Offset localPosition = box.globalToLocal(
                                  details.globalPosition,
                                );
                                final x = _convertToChartX(
                                  localPosition.dx,
                                  constraints.maxWidth,
                                );
                                final y = _convertToChartY(
                                  localPosition.dy,
                                  constraints.maxHeight,
                                );
                                _draggedPointIndex = _findNearestPointIndex(
                                  x,
                                  y,
                                );
                                if (_draggedPointIndex != null) {
                                  _selectedPointIndex = _draggedPointIndex;
                                  setState(() {});
                                }
                              }
                            : null,
                        onPanUpdate: !widget.readOnly
                            ? (details) {
                                if (_draggedPointIndex != null) {
                                  RenderBox box =
                                      context.findRenderObject() as RenderBox;
                                  Offset localPosition = box.globalToLocal(
                                    details.globalPosition,
                                  );
                                  final x = _convertToChartX(
                                    localPosition.dx,
                                    constraints.maxWidth,
                                  );
                                  final y = _convertToChartY(
                                    localPosition.dy,
                                    constraints.maxHeight,
                                  );
                                  _updatePointPosition(
                                    _draggedPointIndex!,
                                    x,
                                    y,
                                  );
                                }
                              }
                            : null,
                        onPanEnd: !widget.readOnly
                            ? (details) {
                                _draggedPointIndex = null;
                                setState(() {});
                              }
                            : null,
                        child: LineChart(
                          LineChartData(
                            lineBarsData: [
                              LineChartBarData(
                                spots: _points
                                    .map((p) => FlSpot(p.tempC, p.speedPercent))
                                    .toList(),
                                isCurved: false,
                                barWidth: 4,
                                color: Theme.of(context).colorScheme.primary,
                                dotData: FlDotData(
                                  show: true,
                                  getDotPainter:
                                      (spot, percent, barData, index) {
                                        final colorScheme = Theme.of(
                                          context,
                                        ).colorScheme;
                                        return FlDotCirclePainter(
                                          radius: 8,
                                          color: _draggedPointIndex == index
                                              ? colorScheme.secondary
                                              : _selectedPointIndex == index
                                              ? colorScheme.tertiary
                                              : colorScheme.primary,
                                          strokeWidth:
                                              _selectedPointIndex == index
                                              ? 4
                                              : 2,
                                          strokeColor: colorScheme.surface,
                                        );
                                      },
                                ),
                              ),
                            ],
                            lineTouchData: LineTouchData(enabled: false),
                            gridData: FlGridData(
                              show: true,
                              drawVerticalLine: true,
                              drawHorizontalLine: true,
                              verticalInterval: 10,
                              horizontalInterval: 10,
                              getDrawingHorizontalLine: (value) {
                                return FlLine(
                                  color: Theme.of(
                                    context,
                                  ).colorScheme.outline.withValues(alpha: 0.2),
                                  strokeWidth: 1,
                                );
                              },
                              getDrawingVerticalLine: (value) {
                                return FlLine(
                                  color: Theme.of(
                                    context,
                                  ).colorScheme.outline.withValues(alpha: 0.2),
                                  strokeWidth: 1,
                                );
                              },
                            ),
                            titlesData: FlTitlesData(
                              show: true,
                              rightTitles: AxisTitles(
                                sideTitles: SideTitles(showTitles: false),
                              ),
                              topTitles: AxisTitles(
                                sideTitles: SideTitles(showTitles: false),
                              ),
                              bottomTitles: AxisTitles(
                                sideTitles: SideTitles(
                                  showTitles: true,
                                  reservedSize: 30,
                                  interval: 10,
                                  getTitlesWidget: (value, meta) {
                                    if (value >= 20 &&
                                        value <= 100 &&
                                        value % 10 == 0) {
                                      return Text(
                                        '${value.toInt()}°C',
                                        style: Theme.of(
                                          context,
                                        ).textTheme.bodySmall,
                                      );
                                    }
                                    return const SizedBox.shrink();
                                  },
                                ),
                              ),
                              leftTitles: AxisTitles(
                                sideTitles: SideTitles(
                                  showTitles: true,
                                  reservedSize: 40,
                                  interval: 10,
                                  getTitlesWidget: (value, meta) {
                                    if (value >= 0 &&
                                        value <= 100 &&
                                        value % 10 == 0) {
                                      return Text(
                                        '${value.toInt()}%',
                                        style: Theme.of(
                                          context,
                                        ).textTheme.bodySmall,
                                      );
                                    }
                                    return const SizedBox.shrink();
                                  },
                                ),
                              ),
                            ),
                            borderData: FlBorderData(
                              show: true,
                              border: Border.all(
                                color: Theme.of(
                                  context,
                                ).colorScheme.outline.withValues(alpha: 0.3),
                              ),
                            ),
                            minX: 20,
                            maxX: 100,
                            minY: 0,
                            maxY: 100,
                            extraLinesData: ExtraLinesData(
                              horizontalLines: [
                                if (widget.currentSpeed != null)
                                  HorizontalLine(
                                    y: widget.currentSpeed!,
                                    color: Colors.orange,
                                    strokeWidth: 2,
                                    dashArray: [5, 5],
                                  ),
                              ],
                              verticalLines: [
                                if (widget.currentTemp != null)
                                  VerticalLine(
                                    x: widget.currentTemp!,
                                    color: Colors.orange,
                                    strokeWidth: 2,
                                    dashArray: [5, 5],
                                  ),
                              ],
                            ),
                          ),
                        ),
                      );
                    },
                  ),
                ),
                const SizedBox(height: 16),
                if (_selectedPointIndex != null) ...[
                  Text(
                    _selectedPointLabel(),
                    style: Theme.of(context).textTheme.bodyMedium,
                  ),
                  const SizedBox(height: 8),
                ],
                Text(
                  'Temperature (°C) → Fan Speed (%)',
                  style: Theme.of(context).textTheme.bodySmall?.copyWith(
                    color: Theme.of(context).colorScheme.onSurfaceVariant,
                  ),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }

  String _selectedPointLabel() {
    final index = _selectedPointIndex;
    if (index == null || index < 0 || index >= _points.length) {
      return 'No point selected';
    }
    final point = _points[index];
    return 'Point ${index + 1}: ${point.tempC.round()}°C / ${point.speedPercent.round()}%';
  }

  void _handleUiInputCommand(UiInputCommand command) {
    if (widget.readOnly || _points.isEmpty) {
      return;
    }

    final current = _selectedPointIndex ?? 0;
    switch (command) {
      case UiInputCommand.curvePreviousPoint:
        setState(() {
          _selectedPointIndex = (current - 1).clamp(0, _points.length - 1);
        });
      case UiInputCommand.curveNextPoint:
        setState(() {
          _selectedPointIndex = (current + 1).clamp(0, _points.length - 1);
        });
      case UiInputCommand.curveSpeedUp:
        _nudgeSelectedPoint(deltaSpeed: 1);
      case UiInputCommand.curveSpeedDown:
        _nudgeSelectedPoint(deltaSpeed: -1);
      case UiInputCommand.curveTempLeft:
        _nudgeSelectedPoint(deltaTemp: -1);
      case UiInputCommand.curveTempRight:
        _nudgeSelectedPoint(deltaTemp: 1);
      case UiInputCommand.focusUp:
      case UiInputCommand.focusDown:
      case UiInputCommand.focusLeft:
      case UiInputCommand.focusRight:
      case UiInputCommand.activate:
      case UiInputCommand.back:
      case UiInputCommand.apply:
      case UiInputCommand.nextTab:
      case UiInputCommand.previousTab:
        return;
    }
  }

  void _nudgeSelectedPoint({double deltaTemp = 0, double deltaSpeed = 0}) {
    final index = _selectedPointIndex ?? 0;
    _selectedPointIndex = index;
    final point = _points[index];
    _updatePointPosition(
      index,
      point.tempC + deltaTemp,
      point.speedPercent + deltaSpeed,
    );
  }

  double _convertToChartX(double pixelX, double width) {
    // Chart X range: 20-100
    // Chart has left margin of 40 for axis labels
    // Convert pixel position to chart value
    final chartWidth = width - 40; // Subtract left margin
    final chartX = pixelX - 40; // Subtract left margin from pixel position
    return 20 + (chartX / chartWidth) * (100 - 20);
  }

  double _convertToChartY(double pixelY, double height) {
    // Chart Y range: 0-100
    // Chart has bottom margin of 30 for axis labels
    // Convert pixel position to chart value (inverted since Y grows downward)
    final chartHeight = height - 30; // Subtract bottom margin
    final chartY = pixelY; // No offset needed for top margin
    return 100 - (chartY / chartHeight) * 100;
  }

  int? _findNearestPointIndex(double x, double y) {
    if (_points.isEmpty) return null;

    int? nearestIndex;
    double minDistance = double.infinity;

    for (int i = 0; i < _points.length; i++) {
      final distance = _calculateDistance(
        x,
        y,
        _points[i].tempC,
        _points[i].speedPercent,
      );
      if (distance < minDistance) {
        minDistance = distance;
        nearestIndex = i;
      }
    }

    return minDistance < 200
        ? nearestIndex
        : null; // Increased to 200 for easier selection
  }

  double _calculateDistance(double x1, double y1, double x2, double y2) {
    return ((x1 - x2) * (x1 - x2) + (y1 - y2) * (y1 - y2)).abs();
  }

  void _updatePointPosition(int index, double x, double y) {
    setState(() {
      var newTemp = x.clamp(20.0, 100.0);
      final newSpeed = y.clamp(0.0, 100.0);

      if (index > 0) {
        final minTemp = _points[index - 1].tempC.round() + 1;
        newTemp = newTemp.clamp(minTemp.toDouble(), 100.0);
      }
      if (index < _points.length - 1) {
        final maxTemp = _points[index + 1].tempC.round() - 1;
        if (maxTemp >= 20) {
          newTemp = newTemp.clamp(20.0, maxTemp.toDouble());
        }
      }

      _points[index] = FanCurvePoint(tempC: newTemp, speedPercent: newSpeed);

      // If the new Y value is lower than the point to the left,
      // propagate this change to all points to the left
      if (index > 0 && newSpeed < _points[index - 1].speedPercent) {
        for (int i = index - 1; i >= 0; i--) {
          if (_points[i].speedPercent > newSpeed) {
            _points[i] = FanCurvePoint(
              tempC: _points[i].tempC,
              speedPercent: newSpeed,
            );
          } else {
            break; // No need to continue further left
          }
        }
      }

      // If the new Y value is higher than the point to the right,
      // propagate this change to all points to the right
      if (index < _points.length - 1 &&
          newSpeed > _points[index + 1].speedPercent) {
        for (int i = index + 1; i < _points.length; i++) {
          if (_points[i].speedPercent < newSpeed) {
            _points[i] = FanCurvePoint(
              tempC: _points[i].tempC,
              speedPercent: newSpeed,
            );
          } else {
            break; // No need to continue further right
          }
        }
      }

      // Note: Changes are only applied when Apply button is clicked
    });
  }
}

class _FanCurveInputAction extends Action<UiInputIntent> {
  final ValueChanged<UiInputCommand> onCommand;
  final BuildContext parentContext;

  _FanCurveInputAction(this.onCommand, this.parentContext);

  bool _isCurveCommand(UiInputCommand command) {
    return switch (command) {
      UiInputCommand.curvePreviousPoint ||
      UiInputCommand.curveNextPoint ||
      UiInputCommand.curveSpeedUp ||
      UiInputCommand.curveSpeedDown ||
      UiInputCommand.curveTempLeft ||
      UiInputCommand.curveTempRight => true,
      UiInputCommand.focusUp ||
      UiInputCommand.focusDown ||
      UiInputCommand.focusLeft ||
      UiInputCommand.focusRight ||
      UiInputCommand.activate ||
      UiInputCommand.back ||
      UiInputCommand.apply ||
      UiInputCommand.nextTab ||
      UiInputCommand.previousTab => false,
    };
  }

  @override
  Object? invoke(UiInputIntent intent) {
    if (!_isCurveCommand(intent.command)) {
      return Actions.maybeInvoke(parentContext, intent);
    }
    onCommand(intent.command);
    return null;
  }
}
