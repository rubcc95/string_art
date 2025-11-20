import 'dart:js_interop';
import 'dart:ui';
import "models/basic_types.dart" as dart;
import "models/engine.dart" as native;

@JS()
external JSPromise<JSAny?> initWasm();

@JS()
extension type Point._(JSObject _obj) {
  external JSNumber get x;
  external JSNumber get y;

  dart.Point get toDart => dart.Point(x.toDartDouble, y.toDartDouble);
}

@JS()
extension type Segment._(JSObject _obj) {
  external Point get start;
  external Point get end;

  dart.Segment get toDart => dart.Segment(start.toDart, end.toDart);
}

@JS()
extension type Step._(JSObject _obj) {
  external JSNumber get color;
  external JSNumber get link;
  external JSNumber get nail;
  external Segment get segment;

  dart.Step get toDart => dart.Step(
    color: Color(color.toDartInt),
    link: link.toDartInt,
    nail: nail.toDartInt,
    segment: segment.toDart,
  );
}

@JS()
extension type Settings._(JSObject _obj) {
  external factory Settings();

  external JSNumber get decay;
  external set decay(JSNumber v);

  external JSNumber get minNailDistance;
  external set minNailDistance(JSNumber v);

  external JSNumber get nailCount;
  external set nailCount(JSNumber v);

  external JSNumber get circularNailRadius;
  external set circularNailRadius(JSNumber v);

  native.Settings get toDart => native.Settings(
    decay: decay.toDartDouble,
    minNailDistance: minNailDistance.toDartInt,
    nailCount: nailCount.toDartInt,
    circularNailRadius: circularNailRadius.toDartDouble,
  );
}

extension NativeSettings on native.Settings {
  Settings get toJS {
    var js = Settings();
    js.decay = decay.toJS;
    js.minNailDistance = minNailDistance.toJS;
    js.nailCount = nailCount.toJS;
    js.circularNailRadius = circularNailRadius.toJS;
    return js;
  }
}

@JS()
extension type Computation._(JSObject _obj) {
  external factory Computation(Settings settings, JSUint8Array imageBuffer);
  external Step? next();
}

// @JS('RustCounter')
// extension type RustCounter._(JSObject _obj) {
//   external factory RustCounter(JSNumber max);
//   external JSNumber? next();

//   external JSNumber get current;
//   external set current(JSNumber v);

//   external JSNumber get max;
//   external set max(JSNumber v);
// }

// @JS('Computation')
// extension type Computation._(JSObject _obj) {
//   external factory Computation(JSNumber max);
//   external JSObject? next();
// }

// @JS()
// extension type StepRaw(JSObject _obj) {
//   external JSArray get layer;
//   external JSNumber get nail;
//   external JSNumber get link;
//   external Segment get segment;
// }

// @JS()
// extension type Segment._(JSObject _obj) {
//   external Point get a;
//   external Point get b;
// }

// @JS()
// extension type Point._(JSObject _obj) {
//   external JSNumber get x;
//   external JSNumber get y;
// }
