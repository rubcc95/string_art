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
  Future<Pipeline> build(Uint8List image) async {
    return JsPipeline(interop.Pipeline(image.toJS));
  }
}

class JsPipeline extends Pipeline {
  JsPipeline(this._pipeline);
  final interop.Pipeline _pipeline;

  @override
  StepIterator build(Settings settings) {
    return JsStepIterator(_pipeline.build(settings.toJS));
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
