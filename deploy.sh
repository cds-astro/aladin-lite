USER_ALADIN="matthieu.baumann"

ssh $USER_ALADIN@aladin 'sg hips -c "rm -rf $HOME/*"'
scp dist/* $USER_ALADIN@aladin:
ssh $USER_ALADIN@aladin 'sg hips -c "rm -rf /home/thomas.boch/AladinLite/www/api/v3/2022-10-17 && 
mkdir -p /home/thomas.boch/AladinLite/www/api/v3/2022-10-17 && 
cp $HOME/* /home/thomas.boch/AladinLite/www/api/v3/2022-10-17/ && 
rm /home/thomas.boch/AladinLite/www/api/v3/latest && 
ln -s /home/thomas.boch/AladinLite/www/api/v3/2022-10-17/ /home/thomas.boch/AladinLite/www/api/v3/latest"'