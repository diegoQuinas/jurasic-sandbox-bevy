
# Dinosaur simulator


## Roadmap


- [x] 16 systems on chain(), nothing runs in parallel (tons of queries)
- [x] render -> HashMap of ALL entities and redraws the terminal on each Update
- [x] PopulationHistory grows one point on tick without top. Ratatui chart gets heavy
- [x] find_closest_tile Vec::with_capacity(3) per movement
- [x] Not scanning all plants, maybe use chunks "plant near in radio N"
- [x] Filter queries, only SeekingFood / SeekingPartner
- [x] Limit catch-up of Fixed
- [x] Cap or downsample the history of charts
