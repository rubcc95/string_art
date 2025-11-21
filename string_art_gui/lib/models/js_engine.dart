import 'dart:async';
import 'dart:js_interop';
import 'dart:js_interop_unsafe';
import 'dart:ui';

import 'basic_types.dart';
import 'engine.dart';

class JsEngine extends Engine {
  const JsEngine();

  @override
  Stream<Step> build(Settings settings) {
    final controller = StreamController<Step>();
    WebWorker(
      ((JSObject raw) {
        controller.add(raw.toStep);
      }).toJS,
      settings.toJS,
    );
    return controller.stream;
  }
}

extension on JSObject {
  Point get toPoint {
    return Point(0, 0);
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
