import 'package:flutter/material.dart';

void main() {
  runApp(MyApp());
}

class MyApp extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: Text("Tabs en subrecuadro")),
      body: Center(
        child: Container(
          width: 300,
          height: 300,
          decoration: BoxDecoration(
            border: Border.all(color: Colors.blue),
            borderRadius: BorderRadius.circular(10),
          ),
          child: DefaultTabController(
            length: 3, // Número de tabs
            child: Column(
              children: [
                TabBar(
                  labelColor: Colors.blue,
                  unselectedLabelColor: Colors.grey,
                  tabs: [
                    Tab(text: "Tab 1"),
                    Tab(text: "Tab 2"),
                    Tab(text: "Tab 3"),
                  ],
                ),
                Expanded(
                  child: TabBarView(
                    children: [
                      Center(child: Text("Contenido Tab 1")),
                      Center(child: Text("Contenido Tab 2")),
                      Center(child: Text("Contenido Tab 3")),
                    ],
                  ),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}
