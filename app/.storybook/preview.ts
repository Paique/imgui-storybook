import type { Preview } from '@storybook/html-vite';

const preview: Preview = {
  globalTypes: {
    imguiTheme: {
      description: 'Host canvas theme',
      toolbar: {
        title: 'Theme',
        icon: 'mirror',
        items: [
          { value: 'dark', title: 'Dark' },
          { value: 'light', title: 'Light' },
        ],
        dynamicTitle: true,
      },
    },
    imguiScale: {
      description: 'Host UI scale (font/metric scale on the host)',
      toolbar: {
        title: 'Scale',
        icon: 'grow',
        items: [
          { value: '1', title: '1x' },
          { value: '1.5', title: '1.5x' },
          { value: '2', title: '2x' },
        ],
        dynamicTitle: true,
      },
    },
    imguiBackdrop: {
      description: 'What is drawn behind the story window',
      toolbar: {
        title: 'Backdrop',
        icon: 'photo',
        items: [
          { value: 'neutral-dark', title: 'Neutral dark' },
          { value: 'neutral-light', title: 'Neutral light' },
          { value: 'checker', title: 'Checker' },
        ],
        dynamicTitle: true,
      },
    },
    imguiCanvasMode: {
      description: 'How the story is laid out on the canvas',
      toolbar: {
        title: 'Canvas',
        icon: 'component',
        items: [
          { value: 'windowed', title: 'Windowed' },
          { value: 'inline', title: 'Inline' },
        ],
        dynamicTitle: true,
      },
    },
  },
  initialGlobals: {
    imguiTheme: 'dark',
    imguiScale: '1',
    imguiBackdrop: 'neutral-dark',
    imguiCanvasMode: 'windowed',
  },
};

export default preview;
