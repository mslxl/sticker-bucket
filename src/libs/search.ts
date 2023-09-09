import { MemeQueried, searchMeme } from './native/db'

interface SearchOption{
  statement: string
  filterFav: boolean
  filterTrash: boolean
}

export class SearchRequestBuilder implements SearchOption{
  statement: string
  filterFav: boolean
  filterTrash: boolean

  constructor() {
    this.statement = ''
    this.filterFav = false
    this.filterTrash = false
  }

  build(): SearchResponse{
    const res = new SearchResponse(this)
    return res
  }
}

export class SearchResponse{
  chunk: MemeQueried[]
  index: number
  endOfSearch: boolean
  options: SearchOption

  constructor(options: SearchOption){
    this.chunk = []
    this.index = 0
    this.endOfSearch = false
    this.options = options
  }

  data(): MemeQueried[]{
    return this.chunk
  }

  hasNext(): boolean {
    return !this.endOfSearch
  }

  async next(): Promise<SearchResponse> {
    const nextGeneration = new SearchResponse(this.options)
    const addition = await searchMeme(this.options.statement, this.index, this.options.filterFav, this.options.filterTrash)
    nextGeneration.chunk = [...this.chunk, ...addition]
    nextGeneration.index = this.index + 1
    nextGeneration.endOfSearch = addition.length == 0
    return nextGeneration
  }

}