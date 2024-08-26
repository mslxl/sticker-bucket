import{app} from '@tauri-apps/api'
import {capitalize, join, map, pipe, split} from 'lodash/fp'

export interface AppInfo{
    appName: string,
    appVersion: string
}
const makeName = pipe([
    split('-'),
    map(capitalize),
    join(' ')
])

export default async function():Promise<AppInfo>{
    const [name, version] = await Promise.all([app.getName(), app.getVersion()])
    
    return {
        appName: makeName(name),
        appVersion: version
    }
}