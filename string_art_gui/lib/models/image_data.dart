import 'dart:typed_data';

import 'package:string_art_gui/models/computation.dart';

import 'engine.dart';

class ImageData {
  ImageData(this.buffer) : pipeline = Engine.instance.build(buffer);

  final Uint8List buffer;
  final Future<Pipeline> pipeline;
  final List<Computation> computations = [];
}
