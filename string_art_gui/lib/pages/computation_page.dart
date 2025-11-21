import 'dart:async';
import 'package:flutter/material.dart' hide Step;

import '../models/computation.dart';
import '../models/engine.dart';
import '../models/image_data.dart';
import '../models/basic_types.dart';

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

  @override
  void initState() {
    super.initState();

    widget.computation.then((cmp) {
      setState(() => _computation = cmp);
      cmp.listen(
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
        child: Row(
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
              painter: LinePainter(size: _computation!.size, steps: _steps),
            ),
      bottomNavigationBar: _buildBottomButtons(),
    );
  }
}

class LinePainter extends CustomPainter {
  final List<Step> steps;
  final Size size;

  LinePainter({required this.steps, Size? size}) : size = size ?? Size(0, 0);

  @override
  void paint(Canvas canvas, Size size) {
    canvas.scale(size.width / this.size.width, size.height / this.size.height);
    final paint = Paint()
      ..color = Colors.black
      ..strokeWidth = 0.2
      ..style = PaintingStyle.stroke;

    for (var step in steps) {
      canvas.drawLine(step.segment.start, step.segment.end, paint);
    }
  }

  @override
  bool shouldRepaint(covariant CustomPainter oldDelegate) {
    return true;
  }
}
