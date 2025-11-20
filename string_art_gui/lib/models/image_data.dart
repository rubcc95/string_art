import 'dart:typed_data';
import 'package:string_art_gui/models/computation.dart';

class ImageData {
  ImageData(this.buffer);

  final Uint8List buffer;
  final List<Computation> computations = [];
}
