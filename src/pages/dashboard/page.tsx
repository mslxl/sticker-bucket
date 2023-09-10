import { path } from '@tauri-apps/api'
import { useEffect, useMemo, useState } from 'react'

import Box from '@mui/material/Box'
import CssBaseline from '@mui/material/CssBaseline'
import { useLocation, useMatch, useParams } from 'react-router-dom'
import { getStorage } from '../../libs/native/db'
import DashboardAppbar from './DashboardAppbar'
import DashboardDrawer, { DrawerHeader } from './DashboardDrawer'
import { capitalize } from '@mui/material'
import DashboardContent from './DashboardContent'
import { SearchRequestBuilder } from '../../libs/search'
import { AnimatePresence, motion } from 'framer-motion'

export default function DashboardLayout() {

  const [windowTitle, setWindowTitle] = useState('Meme Management')
  useEffect(() => {
    getStorage().then(storage => {
      setWindowTitle(capitalize(storage.substring(storage.lastIndexOf(path.sep) + 1)))
    })
  })

  const params = useParams()
  const [drawerOpen, setDrawerOpen] = useState(false)
  const [searchContent, setSearchContent] = useState(params.search || '')
  const favMatch = useMatch('/dashboard/fav')
  const trashMatch = useMatch('/dashboard/trash')

  const searchResponse = useMemo(() => {
    const builder = new SearchRequestBuilder()
    builder.statement = searchContent
    builder.filterFav = favMatch != null
    builder.filterTrash = trashMatch != null
    return builder.build()
  }, [searchContent, favMatch, trashMatch])

  const location = useLocation()


  return (
    <AnimatePresence mode='wait'>
      <Box sx={{ display: 'flex' }} key={location.pathname}>
        <CssBaseline />
        <DashboardAppbar
          title={windowTitle}
          drawerOpen={drawerOpen}
          onDrawerToggle={setDrawerOpen}
          onSearchValueChange={setSearchContent}
          defaultSearchValue={searchContent} />
        <DashboardDrawer
          open={drawerOpen}
          onOpenChange={setDrawerOpen} />
        <Box component='main' sx={{ flexGrow: 1, p: 3 }}>
          <DrawerHeader />
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}>
            <DashboardContent initialSearchResponse={searchResponse} />
          </motion.div>
        </Box>
      </Box>
    </AnimatePresence>
  )
}