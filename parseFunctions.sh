grep ".[a-zA-Z]* = function" healpix.js | while read line; do
p=$(echo $(echo $line | cut -f1 -d ' '))
if [[ $p == *"."* ]]; then
echo $p
fi
done > hpx.out
while read line; do
s=$(echo $(echo $line | rev | cut -f1 -d '.' | rev))
echo $s
done < hpx.out > funcs.out


while read p; do
echo $p
res=find . -type f | xargs grep $p
done < ../../../funcs.out
