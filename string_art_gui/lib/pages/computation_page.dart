import 'dart:async';
import 'dart:math' as math;
import 'package:flutter/material.dart' hide Step;

import '../models/engine.dart';
import '../models/image_data.dart';
import '../models/basic_types.dart';
import '../models/draw_backend.dart';

class ComputationPage extends StatefulWidget {
  const ComputationPage(this.image, this.computation, {super.key});

  final ImageData image;
  final Future<Computation> computation;

  @override
  State<ComputationPage> createState() => _ComputationPageState();
}

class _ComputationPageState extends State<ComputationPage> {
  Computation? _computation;
  late final StreamSubscription<Step> _stepSub;
  final List<Step> _steps = [];
  int? _index;

  @override
  void initState() {
    super.initState();

    widget.computation.then((cmp) {
      setState(() => _computation = cmp);
      _stepSub = cmp.listen(
        (step) => setState(() => _steps.add(step)),
        onError: (err, st) {
          print('Computation error: $err');
        },
        onDone: () {
          print('Computation finished.');
        },
        cancelOnError: true,
      );
    });
  }

  @override
  void dispose() {
    _stepSub.cancel();
    super.dispose();
  }

  Widget _buildBottomButtons() {
    return SafeArea(
      top: false,
      child: Container(
        padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
        decoration: BoxDecoration(
          color: Theme.of(context).colorScheme.surface,
          boxShadow: const [
            BoxShadow(
              blurRadius: 6,
              color: Colors.black26,
              offset: Offset(0, -2),
            ),
          ],
        ),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            _steps.isNotEmpty
                ? Slider(
                    min: 1,
                    max: _steps.length.toDouble(),
                    value: _index?.toDouble() ?? _steps.length.toDouble(),
                    divisions: _steps.length - 1,
                    onChanged: (event) {
                      setState(() {
                        var value = event.toInt();
                        _index = value == _steps.length ? null : value;
                      });
                    },
                    label: (_index ?? _steps.length).toString(),
                  )
                : const SizedBox(),
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceEvenly,
              children: [
                IconButton(
                  tooltip: 'Acción 1',
                  iconSize: 32,
                  onPressed: () {
                    // TODO: implementar acción 1
                  },
                  icon: const Icon(Icons.play_arrow),
                ),
                IconButton(
                  tooltip: 'Acción 2',
                  iconSize: 32,
                  onPressed: () {
                    // TODO: implementar acción 2
                  },
                  icon: const Icon(Icons.pause),
                ),
                IconButton(
                  tooltip: 'Acción 3',
                  iconSize: 32,
                  onPressed: () {
                    // TODO: implementar acción 3
                  },
                  icon: const Icon(Icons.stop),
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: _computation == null
          ? Center(child: CircularProgressIndicator())
          : CustomPaint(
              size: MediaQuery.of(context).size,
              painter: LinePainter(
                size: _computation!.size,
                steps: _index == null ? _steps : _steps.sublist(0, _index),
              ),
            ),
      bottomNavigationBar: _buildBottomButtons(),
    );
  }
}

class CanvasDrawBackend with DrawBackend {
  Canvas _canvas;
  Paint _paint = Paint();

  CanvasDrawBackend(this._canvas);

  @override
  void drawCircle(Offset offset, double radius, Color color) {
    _paint.color = color;
    _paint.style = PaintingStyle.fill;
    _canvas.drawCircle(offset, radius, _paint);
  }

  @override
  void drawCircunference(
    Offset offset,
    double radius,
    double stroke,
    Color color,
  ) {
    _paint.color = color;
    _paint.style = PaintingStyle.stroke;
    _paint.strokeWidth = stroke;
    _canvas.drawCircle(offset, radius, _paint);
  }

  @override
  void drawSegment(Segment segment, double stroke, Color color) {
    _paint.color = color;
    _paint.style = PaintingStyle.stroke;
    _paint.strokeWidth = stroke;
    _canvas.drawLine(segment.start, segment.end, _paint);
  }
}

class LinePainter extends CustomPainter {
  final List<Step> _steps;
  final Size _size;

  LinePainter({required List<Step> steps, Size? size})
    : _steps = steps,
      _size = size ?? Size(0, 0);

  @override
  void paint(Canvas canvas, Size size) {
    final scale = math.min(
      size.width / this._size.width,
      size.height / this._size.height,
    );
    canvas.scale(scale);
    final paint = Paint()
      ..color = Colors.black
      ..strokeWidth = 0.2
      ..style = PaintingStyle.stroke;

    for (var step in _steps) {
      canvas.drawLine(step.segment.start, step.segment.end, paint);
    }
  }

  @override
  bool shouldRepaint(covariant LinePainter oldDelegate) =>
      oldDelegate._steps.length != _steps.length;
}
