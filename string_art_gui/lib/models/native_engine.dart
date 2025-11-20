import 'dart:typed_data';

import 'package:string_art_gui/models/engine.dart';

class NativeEngine extends Engine {
  const NativeEngine();

  @override
  Future<void> init() async {
    throw UnimplementedError;
  }

  @override
  Future<Pipeline> build(Uint8List image) {
    throw UnimplementedError();
  }
}
