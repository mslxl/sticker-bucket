import { RouterProvider, createBrowserRouter } from "react-router-dom"

import PageMain from './pages/main'

const router = createBrowserRouter([
    {
        path: '/',
        element: <PageMain/>
    }
])


export default function Router(){
    return <RouterProvider router={router}/>

}