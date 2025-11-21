import 'dart:async';
import 'dart:ui';

import 'package:string_art_gui/models/engine.dart';

import 'basic_types.dart';

class NativeEngine extends Engine {
  const NativeEngine();

  @override
  Future<Computation> build(Settings settings) {
    // TODO: implement build
    throw UnimplementedError();
  }
}

class NativeComputation with Stream<Step> implements Computation {
  @override
  StreamSubscription<Step> listen(
    void Function(Step step)? onData, {
    Function? onError,
    void Function()? onDone,
    bool? cancelOnError,
  }) {
    throw UnimplementedError();
  }

  @override
  Size get size => throw UnimplementedError();
}
