import 'package:flutter/foundation.dart';
import 'package:string_art_gui/models/js_engine.dart';
import 'package:string_art_gui/models/native_engine.dart';

import 'basic_types.dart';

abstract class StepIterator {
  Step? next();
}

abstract class Engine {
  static const Engine instance = kIsWeb ? JsEngine() : NativeEngine();

  const Engine();

  Stream<Step> build(Settings settings);
}

class Settings {
  Settings({
    required this.buffer,
    this.decay = 0.15,
    this.minNailDistance = 20,
    this.nailCount = 512,
    this.circularNailRadius = 0.3,
  });

  double decay;
  int minNailDistance;
  int nailCount;
  double circularNailRadius;
  Uint8List buffer;
}
