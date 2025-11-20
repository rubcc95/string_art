import 'dart:js_interop';
import 'dart:typed_data';

import 'package:string_art_gui/models/basic_types.dart';
import 'package:string_art_gui/models/engine.dart';
import 'package:string_art_gui/rust_interop.dart' as interop;

class JsEngine extends Engine {
  const JsEngine();

  @override
  Future<void> init() async {
    await interop.initWasm().toDart;
  }

  @override
  Future<StepIterator> build(Settings settings, Uint8List image) async {
    return JsStepIterator(interop.Computation(settings.toJS, image.toJS));
  }
}

class JsStepIterator extends StepIterator {
  JsStepIterator(this._computation);

  final interop.Computation _computation;

  @override
  Step? next() {
    return _computation.next()?.toDart;
  }
}
