import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

typedef Int = int;

typedef Double = double;

class SliderNumFieldForm<T extends num> extends StatefulWidget {
  final T min;
  final T max;
  final T? step;
  final T? initialValue;
  final void Function(T)? onChange;
  final List<TextInputFormatter> formatters;
  final T? Function(String) parser;
  final Int fractionDigits;
  final String? label;
  final SliderNumController<T>? controller;

  const SliderNumFieldForm({
    this.label,
    super.key,
    required this.min,
    required this.max,
    this.step,
    this.initialValue,
    this.onChange,
    required this.parser,
    this.formatters = const [],
    this.fractionDigits = 0,
    this.controller,
  });

  static SliderNumFieldForm<Int> int({
    Key? key,
    required Int min,
    required Int max,
    Int? step,
    Int? initialValue,
    String? label,
    void Function(Int)? onChange,
    List<TextInputFormatter> formatters = const [],
    SliderNumController<Int>? controller,
  }) {
    return SliderNumFieldForm<Int>(
      key: key,
      min: min,
      max: max,
      step: step,
      initialValue: initialValue,
      onChange: onChange,
      parser: (s) => Int.tryParse(s),
      formatters: [FilteringTextInputFormatter.digitsOnly, ...formatters],
      label: label,
      controller: controller,
    );
  }

  static SliderNumFieldForm<Double> double({
    Key? key,
    required Double min,
    required Double max,
    String? label,
    Double? step,
    Double? initialValue,
    void Function(Double)? onChange,
    List<TextInputFormatter> formatter = const [],
    Int fractionDigits = 2,
    SliderNumController<Double>? controller,
  }) {
    return SliderNumFieldForm<Double>(
      key: key,
      min: min,
      max: max,
      step: step,
      initialValue: initialValue,
      onChange: onChange,
      fractionDigits: fractionDigits,
      parser: (s) {
        // Accept comma or dot as decimal separator
        final normalized = s.replaceAll(',', '.');
        return Double.tryParse(normalized);
      },
      formatters: [
        // Allow digits, decimal separators (dot/comma) and optional leading minus sign
        FilteringTextInputFormatter.allow(RegExp(r'^[0-9]*\.?[0-9]*$')),
        ...formatter,
      ],
      label: label,
      controller: controller,
    );
  }

  @override
  State<SliderNumFieldForm<T>> createState() => _SliderNumFieldFormState<T>();
}

class _SliderNumFieldFormState<T extends num>
    extends State<SliderNumFieldForm<T>> {
  late Double _value;
  final _textController = TextEditingController();
  late final SliderNumController? _controller;

  @override
  void initState() {
    super.initState();
    _value = widget.initialValue?.toDouble() ?? widget.min.toDouble();
    _textController.text = _value.toStringAsFixed(widget.fractionDigits);
    _controller = widget.controller;
    if (_controller != null) {
      if (_controller._state != null)
        throw Exception(
          "SliderNumController is already attached to another SliderNumFieldForm.",
        );

      _controller._state = this;
    }
  }

  void dispose() {
    _controller?._state = null;

    super.dispose();
  }

  void _updateFromSlider(Double newValue) {
    setState(() {
      _value = newValue;
      _textController.text = newValue.toStringAsFixed(widget.fractionDigits);
    });
  }

  void _updateFromTextField(String text) {
    if (text.isEmpty) return;

    T? newValue = widget.parser(text);
    if (newValue == null) return;

    if (newValue < widget.min) newValue = widget.min;
    if (newValue > widget.max) newValue = widget.max;

    setState(() {
      _value = newValue!.toDouble();
      _textController.value = TextEditingValue(
        text: newValue.toStringAsFixed(widget.fractionDigits),
        selection: TextSelection.collapsed(offset: newValue.toString().length),
      );
    });
  }

  @override
  Widget build(BuildContext context) {
    final divisions = widget.step == null
        ? null
        : ((widget.max - widget.min) / (widget.step!)).ceil();

    print(
      "Divs: $divisions with max: ${widget.max}, min: ${widget.min}, step: ${widget.step}",
    );
    return Row(
      children: [
        if (widget.label != null) Text(widget.label!),
        Expanded(
          child: Slider(
            value: _value.toDouble(),
            min: widget.min.toDouble(),
            max: widget.max.toDouble(),
            divisions: divisions,
            label: _value.toStringAsFixed(widget.fractionDigits),
            onChanged: _updateFromSlider,
          ),
        ),
        SizedBox(
          width: 70,
          child: TextField(
            controller: _textController,
            keyboardType: TextInputType.number,
            inputFormatters: widget.formatters,
            onSubmitted: _updateFromTextField,
          ),
        ),
      ],
    );
  }
}

class SliderNumController<T extends num> {
  SliderNumController();

  _SliderNumFieldFormState<T>? _state;

  void set value(T newValue) {
    if (_state != null) {
      _state!._updateFromSlider(newValue.toDouble());
    }
  }
}
