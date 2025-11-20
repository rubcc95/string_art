import 'package:flutter/material.dart';

class Point {
  Point(this.x, this.y);

  Point.squared(double v) : x = v, y = v;

  double x;
  double y;
}

class Segment {
  Segment(this.start, this.end);

  Point start;
  Point end;
}

class Step {
  Step({
    required this.color,
    required this.link,
    required this.nail,
    required this.segment,
  });

  Color color;
  int link;
  int nail;
  Segment segment;
}
