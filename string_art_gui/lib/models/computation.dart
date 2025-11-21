import 'package:string_art_gui/models/engine.dart';

import 'basic_types.dart';

class Computation {
  Computation(Settings settings)
    : stepListener = Engine.instance.build(settings);

  final Stream<Step> stepListener;
}
