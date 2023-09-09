import { alpha } from '@mui/material'
import { styled } from '@mui/material/styles'
import MuiAppBar, { AppBarProps as MuiAppBarProps } from '@mui/material/AppBar'
import { Toolbar, InputBase } from '@mui/material'
import IconButton from '@mui/material/IconButton'
import { Typography } from '@mui/material'
import {
  Menu as MenuIcon,
  Search as SearchIcon,
} from '@mui/icons-material'
import { drawerWidth } from './DashboardDrawer'

interface AppBarProps extends MuiAppBarProps {
  open?: boolean
}


const AppBar = styled(MuiAppBar, {
  shouldForwardProp: (prop) => prop !== 'open',
})<AppBarProps>(({ theme, open }) => ({
  zIndex: theme.zIndex.drawer + 1,
  transition: theme.transitions.create(['width', 'margin'], {
    easing: theme.transitions.easing.sharp,
    duration: theme.transitions.duration.leavingScreen,
  }),
  ...(open && {
    marginLeft: drawerWidth,
    width: `calc(100% - ${drawerWidth}px)`,
    transition: theme.transitions.create(['width', 'margin'], {
      easing: theme.transitions.easing.sharp,
      duration: theme.transitions.duration.enteringScreen,
    }),
  }),
}))

const Search = styled('div')(({ theme }) => ({
  position: 'relative',
  borderRadius: theme.shape.borderRadius,
  backgroundColor: alpha(theme.palette.common.white, 0.15),
  '&:hover': {
    backgroundColor: alpha(theme.palette.common.white, 0.25),
  },
  marginLeft: 0,
  width: '100%',
  [theme.breakpoints.up('sm')]: {
    marginLeft: theme.spacing(1),
    width: 'auto',
  },
}))

const SearchIconWrapper = styled('div')(({ theme }) => ({
  padding: theme.spacing(0, 2),
  height: '100%',
  position: 'absolute',
  pointerEvents: 'none',
  display: 'flex',
  alignItems: 'center',
  justifyContent: 'center',
}))

const StyledInputBase = styled(InputBase)(({ theme }) => ({
  color: 'inherit',
  '& .MuiInputBase-input': {
    padding: theme.spacing(1, 1, 1, 0),
    // vertical padding + font size from searchIcon
    paddingLeft: `calc(1em + ${theme.spacing(4)})`,
    transition: theme.transitions.create('width'),
    width: '100%',
    [theme.breakpoints.up('sm')]: {
      width: '12ch',
      '&:focus': {
        width: '20ch',
      },
    },
  },
}))

export interface DashboardAppbarProps {
  title: string
  drawerOpen?: boolean
  onDrawerToggle?: (value: boolean) => void
  defaultSearchValue?: string
  onSearchValueChange?: (value: string) => void
}

export default function DashboardAppbar(props: DashboardAppbarProps) {
  return (
    <AppBar position='fixed' open={props.drawerOpen}>
      <Toolbar>
        <IconButton
          color='inherit'
          aria-label='open drawer'
          onClick={() => props.onDrawerToggle && props.onDrawerToggle(!props.drawerOpen)}
          edge='start'
          sx={{
            marginRight: 5,
            ...(props.drawerOpen && { display: 'none' }),
          }}
        >
          <MenuIcon />
        </IconButton>
        <Typography
          variant='h6'
          noWrap
          component='div'
          sx={{ flexGrow: '1' }}>
          {props.title}
        </Typography>

        <Search>
          <SearchIconWrapper>
            <SearchIcon />
          </SearchIconWrapper>
          <StyledInputBase
            placeholder='Search…'
            inputProps={{ 'aria-label': 'search' }}
            defaultValue={props.defaultSearchValue}
            onChange={(e) => props.onSearchValueChange && props.onSearchValueChange(e.target.value)}
          />
        </Search>
      </Toolbar>
    </AppBar>
  )
}