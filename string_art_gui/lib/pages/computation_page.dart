import 'dart:async';
import 'package:flutter/material.dart' hide Step;

import '../models/computation.dart';
import '../models/image_data.dart';
import '../models/basic_types.dart' show Step;

/// Page that displays a full-size image while a computation runs.
/// At the bottom it shows three icon buttons. On each computation [Step]
/// it calls [_callWhenStep].
class ComputationPage extends StatefulWidget {
  const ComputationPage(this.image, this.computation, {super.key});

  final ImageData image;
  final Computation computation;

  @override
  State<ComputationPage> createState() => _ComputationPageState();
}

class _ComputationPageState extends State<ComputationPage> {
  StreamSubscription<Step>? _stepSub;
  Step? _lastStep;

  @override
  void initState() {
    super.initState();
    _listenSteps();
  }

  void _listenSteps() {
    _stepSub = widget.computation.stepListener.listen(
      (step) {
        _lastStep = step;
        _callWhenStep(context, step);
      },
      onError: (err, st) {
        print('Computation error: $err');
      },
      onDone: () {
        print('Computation finished.');
      },
      cancelOnError: false,
    );
  }

  @override
  void dispose() {
    _stepSub?.cancel();
    super.dispose();
  }

  void _callWhenStep(BuildContext context, Step step) {
    print('''Received step: {
      link: ${step.link},
      nail: ${step.link},
      color: ${step.color},
      segment: {
        start: {
          x: ${step.segment.start.x},
          y: ${step.segment.start.y}
        },
        end: {
          x: ${step.segment.end.x},
          y: ${step.segment.end.y}
        }
      }
    }''');
  }

  Widget _buildImageArea() {
    return const Center(
      child: Text('FULL IMAGE HERE', style: TextStyle(fontSize: 18)),
    );
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
      body: _buildImageArea(),
      bottomNavigationBar: _buildBottomButtons(),
    );
  }
}
