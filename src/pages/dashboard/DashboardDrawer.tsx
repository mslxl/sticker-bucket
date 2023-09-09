import { ReactNode } from 'react'
import { styled, useTheme, Theme, CSSObject } from '@mui/material/styles'

import { Drawer as MuiDrawer } from '@mui/material'
import IconButton from '@mui/material/IconButton'
import { Divider } from '@mui/material'
import { List, ListItem, ListItemButton, ListItemIcon, ListItemText } from '@mui/material'
import {
  ChevronLeft as ChevronLeftIcon,
  ChevronRight as ChevronRightIcon,
  AllInbox as AllIcon,
  Favorite as FavoriteIcon,
  Delete as DeleteIcon,
} from '@mui/icons-material'
import { useNavigate } from 'react-router-dom'

export const drawerWidth = 240

const openedMixin = (theme: Theme): CSSObject => ({
  width: drawerWidth,
  transition: theme.transitions.create('width', {
    easing: theme.transitions.easing.sharp,
    duration: theme.transitions.duration.enteringScreen,
  }),
  overflowX: 'hidden',
})

const closedMixin = (theme: Theme): CSSObject => ({
  transition: theme.transitions.create('width', {
    easing: theme.transitions.easing.sharp,
    duration: theme.transitions.duration.leavingScreen,
  }),
  overflowX: 'hidden',
  width: `calc(${theme.spacing(7)} + 1px)`,
  [theme.breakpoints.up('sm')]: {
    width: `calc(${theme.spacing(8)} + 1px)`,
  },
})

export const DrawerHeader = styled('div')(({ theme }) => ({
  display: 'flex',
  alignItems: 'center',
  justifyContent: 'flex-end',
  padding: theme.spacing(0, 1),
  // necessary for content to be below app bar
  ...theme.mixins.toolbar,
}))


const Drawer = styled(MuiDrawer, { shouldForwardProp: (prop) => prop !== 'open' })(
  ({ theme, open }) => ({
    width: drawerWidth,
    flexShrink: 0,
    whiteSpace: 'nowrap',
    boxSizing: 'border-box',
    ...(open && {
      ...openedMixin(theme),
      '& .MuiDrawer-paper': openedMixin(theme),
    }),
    ...(!open && {
      ...closedMixin(theme),
      '& .MuiDrawer-paper': closedMixin(theme),
    }),
  }),
)


interface DrawerItemButtonProp {
  title: string,
  open: boolean,
  onClick?: () => void,
  children: ReactNode,
}
function DrawerItemButton({ title, open, children, onClick }: DrawerItemButtonProp) {
  return (
    <ListItem key={title} disablePadding sx={{ display: 'block' }}>
      <ListItemButton
        sx={{
          minHeight: 48,
          justifyContent: open ? 'initial' : 'center',
          px: 2.5
        }}
        onClick={onClick}
      >
        <ListItemIcon
          sx={{
            minWidth: 0,
            mr: open ? 3 : 'auto',
            justifyContent: 'center'
          }}
        >
          {children}
        </ListItemIcon>
        <ListItemText primary={title} sx={{ opacity: open ? 1 : 0 }} />
      </ListItemButton>
    </ListItem>
  )
}

export interface DashboardDrawerProps{
  open: boolean
  onOpenChange: (value: boolean)=>void
}

export default function DashboardDrawer(props: DashboardDrawerProps) {
  const navgiate = useNavigate()
  const theme = useTheme()
  return (
    <Drawer variant='permanent' open={props.open}>
      <DrawerHeader>
        <IconButton onClick={()=>props.onOpenChange && props.onOpenChange(!props.open)}>
          {theme.direction === 'rtl' ? <ChevronRightIcon /> : <ChevronLeftIcon />}
        </IconButton>
      </DrawerHeader>
      <Divider />

      <List>
        <DrawerItemButton
          title='All'
          onClick={() => { navgiate('/dashboard') }}
          open={props.open}>
          <AllIcon />
        </DrawerItemButton>

        <DrawerItemButton
          title='Favorites'
          onClick={() => { navgiate('/dashboard/fav') }}
          open={props.open}>
          <FavoriteIcon />
        </DrawerItemButton>

        <DrawerItemButton
          title='Trash'
          onClick={() => { navgiate('/dashboard/trash') }}
          open={props.open}>
          <DeleteIcon />
        </DrawerItemButton>
      </List>
      <Divider />
    </Drawer>
  )
}