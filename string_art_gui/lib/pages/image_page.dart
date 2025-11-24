import 'package:flutter/material.dart';
import 'package:string_art_gui/models/image_data.dart';
import 'package:string_art_gui/pages/computation_page.dart';
import 'package:string_art_gui/widgets/slider.dart';

import '../models/engine.dart';

class ImagePage extends StatefulWidget {
  const ImagePage(this.image, {super.key});

  final ImageData image;

  @override
  State<ImagePage> createState() => _ImagePageState();
}

class _ImagePageState extends State<ImagePage> {
  late Settings _settings;
  final _minNailDistanceController = SliderNumController<int>();

  @override
  void initState() {
    super.initState();
    _settings = Settings(buffer: widget.image.buffer);
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(),
      body: Column(
        children: [
          Expanded(
            child: SingleChildScrollView(
              padding: const EdgeInsets.all(16),
              child: Column(
                spacing: 16,
                children: [
                  Row(
                    spacing: 16,
                    children: [
                      Expanded(
                        child: SliderNumFieldForm.double(
                          min: 0.05,
                          max: 5,
                          initialValue: 0.5,
                          label: "Nail Radius",
                          onChange: (d) => _settings.circularNailRadius = d,
                        ),
                      ),
                      Expanded(
                        child: SliderNumFieldForm.int(
                          min: 2,
                          max: 1024,
                          initialValue: 512,
                          label: "Nail Count",
                          step: 1,
                          onChange: (d) {
                            _settings.nailCount = d;
                            _minNailDistanceController.value = (d - 1) ~/ 2;
                          },
                        ),
                      ),
                      Expanded(
                        child: SliderNumFieldForm.int(
                          min: 0,
                          max: 1024,
                          initialValue: 512,
                          label: "Min nail distance",
                          step: 1,
                          controller: _minNailDistanceController,
                          onChange: (d) => _settings.minNailDistance,
                        ),
                      ),
                      Expanded(
                        child: SliderNumFieldForm.double(
                          min: 0.0,
                          max: 1.0,
                          initialValue: 0.1,
                          label: "Decay",
                        ),
                      ),
                    ],
                  ),
                  for (int i = 4; i < 20; i++)
                    TextField(
                      decoration: InputDecoration(
                        labelText: "Field $i",
                        border: OutlineInputBorder(),
                      ),
                    ),
                ],
              ),
            ),
          ),
          SizedBox(
            height: 120,
            child: ListView.separated(
              scrollDirection: Axis.horizontal,
              padding: const EdgeInsets.all(12),
              itemCount: widget.image.computations.length,
              separatorBuilder: (_, __) => const SizedBox(width: 12),
              itemBuilder: (context, index) {
                final computation = widget.image.computations[index];
                return InkWell(
                  onTap: () => Navigator.of(context).push(
                    MaterialPageRoute(
                      builder: (context) => ComputationPage(
                        widget.image,
                        computation,
                        // Engine.instance.build(
                        //   Settings(buffer: widget.image.buffer),
                        // ),
                      ),
                    ),
                  ),
                  child: Container(
                    width: 100,
                    decoration: BoxDecoration(
                      color: Colors.grey[300],
                      borderRadius: BorderRadius.circular(12),
                      image: DecorationImage(
                        image: MemoryImage(widget.image.buffer),
                        fit: BoxFit.contain,
                      ),
                    ),
                  ),
                );
              },
            ),
          ),
        ],
      ),
    );
  }
}
