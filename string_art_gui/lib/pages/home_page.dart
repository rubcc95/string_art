import 'package:flutter/material.dart';
import 'package:image_picker/image_picker.dart';

import '../models/image_data.dart';
import 'image_page.dart';

class HomePage extends StatefulWidget {
  const HomePage({Key? key}) : super(key: key);

  @override
  _HomePageState createState() => _HomePageState();
}

class _HomePageState extends State<HomePage> {
  final _picker = ImagePicker();
  final List<ImageData> _images = [];

  void _pickImage() async {
    final pickedFile = await _picker.pickImage(source: ImageSource.gallery);
    if (pickedFile != null) {
      pickedFile.readAsBytes().then(
        (bytes) => setState(() => _images.add(ImageData(bytes))),
      );
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(),
      body: ListView.builder(
        itemCount: _images.length,
        itemBuilder: (ctx, idx) {
          final image = _images[idx];
          return InkWell(
            onTap: () => Navigator.of(
              context,
            ).push(MaterialPageRoute(builder: (context) => ImagePage(image))),
            child: Card(child: Image.memory(image.buffer)),
          );
        },
      ),
      floatingActionButton: FloatingActionButton(
        onPressed: _pickImage,
        tooltip: 'PickFile',
        child: Icon(Icons.add),
      ),
    );
  }
}
