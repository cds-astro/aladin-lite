USER_ALADIN="matthieu.baumann"
DATEUPLOAD="$(date '+%Y-%m-%d')"

ssh $USER_ALADIN@aladin 'sg hips -c "mkdir -p $HOME/al-tmp && rm -rf $HOME/al-tmp/*"'
scp dist/* $USER_ALADIN@aladin:~/al-tmp

ssh $USER_ALADIN@aladin 'sg hips -c "rm -rf /home/thomas.boch/AladinLite/www/api/v3/24-10-2022d && 
mkdir -p /home/thomas.boch/AladinLite/www/api/v3/24-10-2022d && 
cp $HOME/al-tmp/* /home/thomas.boch/AladinLite/www/api/v3/24-10-2022d && 
rm -rf /home/thomas.boch/AladinLite/www/api/v3/beta && 
ln -s /home/thomas.boch/AladinLite/www/api/v3/24-10-2022d /home/thomas.boch/AladinLite/www/api/v3/beta"'
