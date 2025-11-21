import 'dart:typed_data';
import 'package:string_art_gui/models/computation.dart';
import 'package:string_art_gui/models/engine.dart';

class ImageData {
  ImageData(this.buffer);

  final Uint8List buffer;
  final List<Future<Computation>> computations = [];
}
