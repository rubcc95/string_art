import 'dart:isolate';

import 'package:string_art_gui/models/engine.dart';

import 'basic_types.dart';

class Computation {
  Computation(Pipeline pipeline, Settings settings)
    : stepListener = initializeComputation(pipeline, settings);
  final Stream<Step> stepListener;
}

Stream<Step> initializeComputation(Pipeline pipeline, Settings settings) {
  final rx = ReceivePort();
  // final params = ComputationParams(sendPort: rx.sendPort);
  // final isolate = Isolate.spawn((tx) {}, rx.sendPort);
  // (await isolate).pause
  return rx.asBroadcastStream().cast<Step>();
}

class ComputaionParams {
  const ComputaionParams({
    required this.sendPort,
    required this.iterator,
    required this.pipeline,
  });
  final SendPort sendPort;
  final StepIterator iterator;
  final Pipeline pipeline;
}
