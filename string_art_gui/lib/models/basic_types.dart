import 'package:flutter/material.dart';

class Segment {
  Segment(this.start, this.end);

  Offset start;
  Offset end;
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
