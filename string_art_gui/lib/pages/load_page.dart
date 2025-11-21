import 'package:flutter/material.dart';
import 'home_page.dart';

class LoadPage extends StatefulWidget {
  const LoadPage({Key? key}) : super(key: key);

  @override
  _LoadPageState createState() => _LoadPageState();
}

class _LoadPageState extends State<LoadPage> {
  // bool _isLoaded = false;
  // Error? _error;

  // @override
  // void initState() {
  //   super.initState();

  //   Engine.instance.init().then(
  //     (_) => setState(() => _isLoaded = true),
  //     onError: (err) => setState(() => _error = err),
  //   );
  // }

  @override
  Widget build(BuildContext context) {
    return Scaffold(body: Center(child: HomePage()));
  }
}
