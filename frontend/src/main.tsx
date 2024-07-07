import React from 'react'
import ReactDOM from 'react-dom/client'
import {
  createBrowserRouter,
  RouterProvider,
} from "react-router-dom";
import App from './App.tsx'
import './index.css'
import Today from './today.tsx';
import Last7Days from './last7.tsx';
import Last30Days from './last30.tsx';

const router = createBrowserRouter([
  {
    path: "/",
    element: <App />,
  },
  {
    path: "today",
    element: <Today />,
  },
  {
    path: "last7",
    element: <Last7Days />,
  },
  {
    path: "last30",
    element: <Last30Days />,
  }
]);

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <RouterProvider router={router} />
  </React.StrictMode>,
)
