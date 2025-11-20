import 'dart:typed_data';

import 'package:string_art_gui/models/engine.dart';

class NativeEngine extends Engine {
  const NativeEngine();

  @override
  Future<void> init() async {
    throw UnimplementedError;
  }

  @override
  Future<StepIterator> build(Settings settings, Uint8List image) {
    // TODO: implement build
    throw UnimplementedError();
  }
}
