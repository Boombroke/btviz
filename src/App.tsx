import type { Component } from "solid-js";
import TopBar from "./components/TopBar";
import Canvas from "./components/Canvas";
import Sidebar from "./components/Sidebar";

const App: Component = () => {
  return (
    <div class="flex h-full flex-col bg-surface-0 text-zinc-100">
      <TopBar />
      <main class="relative flex flex-1 min-h-0">
        <Canvas />
        <Sidebar />
      </main>
    </div>
  );
};

export default App;
