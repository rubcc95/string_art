import 'dart:isolate';
import 'dart:typed_data';

import 'package:flutter/foundation.dart';
import 'package:string_art_gui/models/engine.dart';

import 'basic_types.dart';

class Computation {
  Computation(Settings settings, Uint8List imageBuffer)
    : stepListener = initializeComputation(settings, imageBuffer);

  final Stream<Step> stepListener;
}

Stream<Step> initializeComputation(
  Settings settings,
  Uint8List imageBuffer,
) async* {
  final rx = ReceivePort();

  final params = ComputationParams(
    sendPort: rx.sendPort,
    iterator: await Engine.instance.build(settings, imageBuffer),
  );

  compute(ComputationParams.run, params);
  yield* rx.asBroadcastStream().cast<Step>();
}

class ComputationParams {
  const ComputationParams({required this.iterator, required this.sendPort});
  final SendPort sendPort;
  final StepIterator iterator;

  static void run(ComputationParams params) {
    var some;
    while ((some = params.iterator.next()) != null) {
      params.sendPort.send(some);
    }
  }
}
