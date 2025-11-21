import 'package:flutter/material.dart';
import 'pages/load_page.dart';

void main() {
  //exporter = printString.toJS;
  runApp(MyApp());
}

class MyApp extends StatelessWidget {
  // This widget is the root of your application.
  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Flutter Demo',
      theme: ThemeData(primarySwatch: Colors.blue),
      home: LoadPage(),
    );
  }
}
