import 'dart:ui';

import 'package:string_art_gui/models/basic_types.dart';

abstract mixin class DrawBackend {
  void drawSegment(Segment segment, double stroke, Color color);

  void drawCircle(Offset offset, double radius, Color color);

  void drawCircunference(
    Offset offset,
    double radius,
    double stroke,
    Color color,
  );
}
