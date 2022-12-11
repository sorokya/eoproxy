import React from 'react';
import ReactDOM from 'react-dom/client';
import 'agnostic-react/dist/common.min.css';
import 'agnostic-react/dist/esm/index.css';
import { createBrowserRouter, RouterProvider } from 'react-router-dom';
import ErrorPage from './error-page';
import Root from './routes/root';
import App from './routes/app';
import Settings from './routes/settings';

const router = createBrowserRouter([
  {
    path: '/',
    element: <Root />,
    errorElement: <ErrorPage />,
    children: [
      {
        path: '/',
        element: <App />
      },
      {
        path: '/settings',
        element: <Settings />
      }
    ]
  }
]);

const root = ReactDOM.createRoot(document.getElementById('root'));
root.render(
  <RouterProvider router={router} />
);
