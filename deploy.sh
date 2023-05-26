USER_ALADIN="matthieu.baumann"
DATEUPLOAD="$(date '+%Y-%m-%d')"

ssh $USER_ALADIN@aladin 'sg hips -c "mkdir -p /home/matthieu.baumann/al-tmp && rm -rf /home/matthieu.baumann/al-tmp/*"'
# Copy the dist files
scp dist/* $USER_ALADIN@aladin:~/al-tmp
# Copy the tgz
cp aladin-l*.tgz aladin-lite.tgz
scp aladin-lite.tgz $USER_ALADIN@aladin:~/al-tmp

ssh $USER_ALADIN@aladin "sg hips -c 'rm -rf /home/thomas.boch/AladinLite/www/api/v3/$DATEUPLOAD && 
mkdir -p /home/thomas.boch/AladinLite/www/api/v3/$DATEUPLOAD && 
cp /home/matthieu.baumann/al-tmp/* /home/thomas.boch/AladinLite/www/api/v3/$DATEUPLOAD && 
rm -rf /home/thomas.boch/AladinLite/www/api/v3/latest && 
ln -s /home/thomas.boch/AladinLite/www/api/v3/$DATEUPLOAD /home/thomas.boch/AladinLite/www/api/v3/latest'"
