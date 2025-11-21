import 'dart:async';
import 'dart:js_interop';
import 'dart:js_interop_unsafe';
import 'dart:ui';

import 'basic_types.dart';
import 'engine.dart';

class JsEngine extends Engine {
  const JsEngine();

  @override
  Future<Computation> build(Settings settings) async {
    final completer = Completer<Size>();
    final controller = StreamController<Step>();
    WebWorker(
      ((JSObject raw) {
        if (completer.isCompleted) {
          controller.add(raw.toStep);
        } else {
          completer.complete(raw.toSize);
        }
      }).toJS,
      settings.toJS,
    );

    return JsComputation(await completer.future, controller.stream);
  }
}

class JsComputation with Stream<Step> implements Computation {
  JsComputation(this.size, this._stream);

  @override
  final Size size;
  final Stream<Step> _stream;

  @override
  StreamSubscription<Step> listen(
    void Function(Step step)? onData, {
    Function? onError,
    void Function()? onDone,
    bool? cancelOnError,
  }) {
    return _stream.listen(
      onData,
      onDone: onDone,
      onError: onError,
      cancelOnError: cancelOnError,
    );
  }
}

extension on JSObject {
  Offset get toPoint {
    return Offset(
      getProperty<JSNumber>('x'.toJS).toDartDouble,
      getProperty<JSNumber>('y'.toJS).toDartDouble,
    );
  }

  Segment get toSegment {
    return Segment(
      getProperty<JSObject>('start'.toJS).toPoint,
      getProperty<JSObject>('end'.toJS).toPoint,
    );
  }

  Step get toStep {
    return Step(
      color: Color(getProperty<JSNumber>('color'.toJS).toDartInt),
      link: getProperty<JSNumber>('link'.toJS).toDartInt,
      nail: getProperty<JSNumber>('nail'.toJS).toDartInt,
      segment: getProperty<JSObject>('segment'.toJS).toSegment,
    );
  }

  Size get toSize {
    return Size(
      getProperty<JSNumber>('width'.toJS).toDartDouble,
      getProperty<JSNumber>('height'.toJS).toDartDouble,
    );
  }
}

extension on Settings {
  JSObject get toJS {
    var obj = JSObject();
    obj.setProperty('decay'.toJS, decay.toJS);
    obj.setProperty('minNailDistance'.toJS, minNailDistance.toJS);
    obj.setProperty('nailCount'.toJS, nailCount.toJS);
    obj.setProperty('circularNailRadius'.toJS, circularNailRadius.toJS);
    obj.setProperty('buffer'.toJS, buffer.toJS);
    return obj;
  }
}

@JS()
extension type WebWorker._(JSObject _obj) {
  external factory WebWorker(JSFunction message, JSObject settings);
}
