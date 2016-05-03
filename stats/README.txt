cat *txt | grep startup | uniq | grep -v "130.79" | cut -d '  ' -f 1,2  > stats.txt

